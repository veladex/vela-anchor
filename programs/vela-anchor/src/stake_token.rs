use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, TokenAccount};
use crate::{
    constants::*,
    errors::StakeError,
    structs::*,
    events::{StakeCreatedEvent, UnstakedEvent, InterestClaimedEvent, CommunityProfitClaimedEvent},
    referral_utils,
    community_reward,
    zero_copy_storage,
    contexts::{CreateStake, Unstake, ClaimInterest, QueryPendingInterest, QueryCommunityStatus, ClaimCommunityProfit, QueryCurrentRates},
    verify_storage_pdas,
};

// ============ Core functions for production reduction mechanism ============

/// Calculate the current number of production reductions based on total_output
pub fn calc_reduction_count(total_output: u64) -> u16 {
    if total_output >= STAGE_3_END {
        return MAX_REDUCTIONS; // 50
    }
    let count = (total_output / REDUCTION_THRESHOLD) as u16;
    count.min(MAX_REDUCTIONS)
}

/// Calculate the current actual daily rate based on the reduction count and initial basis points value (RATE_BASIS_POINTS precision)
/// Uses cumulative multiplication: after each reduction, rate = current rate * retain_bps / 10000
pub fn calc_current_daily_rate(base_rate_bps: u64, reduction_count: u16) -> u64 {
    let mut rate = base_rate_bps;
    let mut remaining = reduction_count;

    // Stage 1: first 25 times, retain 95% each time
    let stage1_count = remaining.min(STAGE_1_REDUCTIONS);
    for _ in 0..stage1_count {
        rate = rate * STAGE_1_RETAIN_BPS / BASIS_POINTS;
    }
    remaining -= stage1_count;

    if remaining == 0 { return rate.max(1); }

    // Stage 2: next 15 times, retain 97% each time
    let stage2_count = remaining.min(STAGE_2_REDUCTIONS);
    for _ in 0..stage2_count {
        rate = rate * STAGE_2_RETAIN_BPS / BASIS_POINTS;
    }
    remaining -= stage2_count;

    if remaining == 0 { return rate.max(1); }

    // Stage 3: last 10 times, retain 98% each time
    let stage3_count = remaining.min(STAGE_3_REDUCTIONS);
    for _ in 0..stage3_count {
        rate = rate * STAGE_3_RETAIN_BPS / BASIS_POINTS;
    }

    rate.max(1)
}

/// Return the current daily rates for all four tiers (RATE_BASIS_POINTS precision, 1_000_000 = 100%)
pub fn get_current_rates(reduction_count: u16) -> (u64, u64, u64, u64) {
    let rate_7d   = calc_current_daily_rate(DAILY_RATE_7_DAYS, reduction_count);
    let rate_30d  = calc_current_daily_rate(DAILY_RATE_30_DAYS, reduction_count);
    let rate_90d  = calc_current_daily_rate(DAILY_RATE_90_DAYS, reduction_count);
    let rate_365d = calc_current_daily_rate(DAILY_RATE_365_DAYS, reduction_count);
    (rate_7d, rate_30d, rate_90d, rate_365d)
}

/// Get per-address staking cap based on total network staked amount
pub fn get_user_stake_cap(total_staked: u64) -> u64 {
    if total_staked >= TOTAL_STAKED_TIER2 {
        USER_STAKE_CAP_TIER2          // 150,000
    } else if total_staked >= TOTAL_STAKED_TIER1 {
        USER_STAKE_CAP_TIER1          // 100,000
    } else {
        USER_STAKE_CAP_BASE           // 50,000
    }
}

/// Cross-day detection & daily quota update
fn check_and_update_daily_cap(global_state: &mut GlobalState, now: i64) {
    let today = (now as u64) / SECONDS_PER_DAY;

    if today != global_state.current_deposit_day {
        // New day: check if previous day was fully used
        let remaining = global_state.daily_deposit_cap
            .saturating_sub(global_state.daily_deposited);

        if remaining < DAILY_CAP_EXHAUST_THRESHOLD {
            // Previous day was fully used -> new cap = old cap * 110%
            let new_cap = (global_state.daily_deposit_cap as u128)
                .checked_mul(DAILY_CAP_GROWTH_BPS as u128)
                .unwrap()
                .checked_div(BASIS_POINTS as u128)
                .unwrap();

            if new_cap <= u64::MAX as u128 {
                global_state.daily_deposit_cap = new_cap as u64;
            }
            // Keep the original value when exceeding u64::MAX, stop growing
        }
        // Not fully used -> daily_deposit_cap remains unchanged

        // Reset daily counter
        global_state.current_deposit_day = today;
        global_state.daily_deposited = 0;
    }
}

/// Create a new staking order
pub fn handler_create_stake(
    ctx: Context<CreateStake>,
    amount: u64,
    period_type: u8,
) -> Result<()> {
    let user = &ctx.accounts.user;
    let user_stake_account = &mut ctx.accounts.user_stake_account;
    let global_state = &mut ctx.accounts.global_state;
    let wallet_mapping = &ctx.accounts.wallet_mapping;
    let locked_vault = &ctx.accounts.locked_vault;

    // ========== 0. Verify 9 storage PDA addresses (key comparison only) ==========
    verify_storage_pdas!(ctx, global_state);

    // ========== 1. Verify LockedVault PDA ==========
    let token_mint = global_state.stake_token_mint;
    let (expected_locked_vault, _bump) = Pubkey::find_program_address(
        &[LOCKED_VAULT_SEED, token_mint.as_ref()],
        ctx.program_id,
    );
    require!(
        locked_vault.key() == expected_locked_vault,
        StakeError::InvalidLockedVault
    );

    // ========== 2. Verify user has bound a referrer ==========
    require!(
        wallet_mapping.referral_id > 0,
        StakeError::UserNotInReferralSystem
    );

    msg!("User {} has referral_id: {}", user.key(), wallet_mapping.referral_id);

    // ========== 3. Validate stake amount ==========
    // Validate amount range
    require!(
        amount >= MIN_STAKE_AMOUNT && amount <= MAX_STAKE_AMOUNT,
        StakeError::InvalidAmount
    );

    // Must be a whole number of VELA (last 9 digits must be 0)
    require!(
        amount % AMOUNT_DECIMALS == 0,
        StakeError::AmountMustBeWholeNumber
    );

    msg!("Stake amount validated: {} (min: {}, max: {})", amount, MIN_STAKE_AMOUNT, MAX_STAKE_AMOUNT);

    // ========== 3.5 Cross-day detection & update daily quota ==========
    let now = Clock::get()?.unix_timestamp;
    check_and_update_daily_cap(global_state, now);

    // ========== 3.6 Verify daily network-wide quota ==========
    let remaining_cap = global_state.daily_deposit_cap
        .saturating_sub(global_state.daily_deposited);

    require!(
        remaining_cap >= DAILY_CAP_EXHAUST_THRESHOLD,
        StakeError::DailyDepositCapExhausted
    );

    require!(
        amount <= remaining_cap,
        StakeError::DailyDepositCapExceeded
    );

    // ========== 3.7 Verify per-address cap ==========
    let user_cap = get_user_stake_cap(global_state.total_staked);
    let new_total = user_stake_account.total_principal
        .checked_add(amount)
        .ok_or(StakeError::ArithmeticOverflow)?;

    require!(
        new_total <= user_cap,
        StakeError::UserStakeCapExceeded
    );

    msg!("Deposit cap check passed: daily_remaining={}, user_cap={}, user_new_total={}", remaining_cap, user_cap, new_total);

    // ========== 4. Validate period type (using dynamic rates) ==========
    let reduction_count = calc_reduction_count(global_state.total_output);
    let (period_seconds, initial_daily_rate) = match period_type {
        STAKE_PERIOD_7_DAYS  => (PERIOD_7_DAYS,  calc_current_daily_rate(DAILY_RATE_7_DAYS, reduction_count)),
        STAKE_PERIOD_30_DAYS => (PERIOD_30_DAYS, calc_current_daily_rate(DAILY_RATE_30_DAYS, reduction_count)),
        STAKE_PERIOD_90_DAYS  => (PERIOD_90_DAYS,  calc_current_daily_rate(DAILY_RATE_90_DAYS, reduction_count)),
        STAKE_PERIOD_365_DAYS => (PERIOD_365_DAYS, calc_current_daily_rate(DAILY_RATE_365_DAYS, reduction_count)),
        _ => return Err(StakeError::InvalidPeriodType.into()),
    };

    msg!("Staking period: {} (type: {}, seconds: {})", period_type, period_type, period_seconds);

    // ========== 5. Verify order count limit ==========
    require!(
        (user_stake_account.active_count as usize) < MAX_STAKES_PER_USER,
        StakeError::MaxStakesReached
    );

    // ========== 6. Find empty order slot ==========
    let order_index = find_empty_order_slot(&user_stake_account.orders)?;

    msg!("Found empty order slot at index: {}", order_index);

    // ========== 7. Transfer tokens to vault ==========
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.vault_token_account.to_account_info(),
        authority: user.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx, amount)?;

    msg!("Transferred {} tokens from user to vault", amount);

    // ========== 8. Create stake order ==========
    let current_time = Clock::get()?.unix_timestamp;
    let end_time = current_time + period_seconds as i64;

    let order = &mut user_stake_account.orders[order_index];
    order.amount = amount;
    order.period_type = period_type;
    order.start_time = current_time;
    order.end_time = end_time;
    order.last_interest_time = current_time;
    order.accumulated_interest = 0;
    order.claimed_interest = 0;
    order.status = ORDER_STATUS_ACTIVE;
    // Record the daily rate snapshot at creation time (in basis points).
    // The current rate is determined by the global constant for the given period_type.
    // In the future, it may be dynamically adjusted based on TVL, governance votes,
    // or NFT node conditions; interest calculations should then use this field.
    order.initial_daily_rate = initial_daily_rate;
    order.reserved = [0; 5];

    msg!("Created stake order at index {} with amount {}, period {}, initial_daily_rate {} bps, start_time {}, end_time {}",
        order_index, amount, period_type, initial_daily_rate, current_time, end_time);

    // ========== 9. Update user account statistics ==========
    user_stake_account.active_count = user_stake_account.active_count.checked_add(1)
        .ok_or(StakeError::ArithmeticOverflow)?;
    user_stake_account.total_principal = user_stake_account.total_principal.checked_add(amount)
        .ok_or(StakeError::ArithmeticOverflow)?;

    // If first initialization, set owner
    if user_stake_account.owner == Pubkey::default() {
        user_stake_account.owner = user.key();
        user_stake_account.bump = ctx.bumps.user_stake_account;
    }

    msg!("Updated user stats: active_count = {}, total_principal = {}",
        user_stake_account.active_count, user_stake_account.total_principal);

    // ========== 10. Update global state ==========
    global_state.total_staked = global_state.total_staked.checked_add(amount)
        .ok_or(StakeError::ArithmeticOverflow)?;

    // Update daily deposited total
    global_state.daily_deposited = global_state.daily_deposited
        .checked_add(amount)
        .ok_or(StakeError::ArithmeticOverflow)?;

    msg!("Updated global state: total_staked = {}, daily_deposited = {}", global_state.total_staked, global_state.daily_deposited);

    // ========== 10.5. Update staking statistics (today + last 7 days) ==========
    let stats_today = (current_time as u64) / SECONDS_PER_DAY;

    if stats_today > global_state.stats_current_day {
        let days_elapsed = stats_today - global_state.stats_current_day;

        if days_elapsed >= 7 {
            // 超过7天无活动，整个数组清零
            global_state.last_7days_staked = [0; 7];
        } else {
            // 逐天推进，每天左移一位并填零
            for _ in 0..days_elapsed {
                for i in 0..6 {
                    global_state.last_7days_staked[i] = global_state.last_7days_staked[i + 1];
                }
                global_state.last_7days_staked[6] = 0;
            }
        }

        // Reset today's statistics
        global_state.today_staked_amount = 0;
        global_state.stats_current_day = stats_today;

        msg!("Day changed: {} days elapsed, stats_current_day updated to {}", days_elapsed, stats_today);
    }

    // Accumulate today's staked amount
    global_state.today_staked_amount = global_state
        .today_staked_amount
        .checked_add(amount)
        .ok_or(StakeError::ArithmeticOverflow)?;

    // Sync update last_7days_staked[6] (today's real-time value)
    global_state.last_7days_staked[6] = global_state.today_staked_amount;

    msg!("Updated staking stats: today_staked_amount = {}, last_7days_staked[6] = {}",
        global_state.today_staked_amount, global_state.last_7days_staked[6]);

    // ========== 11. Update referral system: accumulate total_staked upward 50 levels ==========
    let storage_accounts = [
        &ctx.accounts.storage_1,
        &ctx.accounts.storage_2,
        &ctx.accounts.storage_3,
        &ctx.accounts.storage_4,
        &ctx.accounts.storage_5,
        &ctx.accounts.storage_6,
        &ctx.accounts.storage_7,
        &ctx.accounts.storage_8,
        &ctx.accounts.storage_9,
    ];

    // Get user's referral_id and parent_id
    let user_referral_id = wallet_mapping.referral_id;
    let (pda_index, slot_index) = ReferralStorage::decode_and_validate_id(user_referral_id)?;

    // Update own self_staked and read parent_id
    let user_parent_id = {
        let storage_account = storage_accounts[(pda_index - 1) as usize];
        let mut storage_data = storage_account.try_borrow_mut_data()?;
        let mut user_referral_data = zero_copy_storage::read_record(&storage_data, slot_index)?;
        let parent_id = user_referral_data.parent_id;
        user_referral_data.self_staked = user_referral_data.self_staked.saturating_add(amount);
        zero_copy_storage::write_record(&mut storage_data, slot_index, &user_referral_data)?;
        parent_id
    }; // storage_data is automatically released here

    msg!("User referral_id: {}, parent_id: {}, self_staked += {}", user_referral_id, user_parent_id, amount);

    // If user has a parent, update 50 levels upward
    if user_parent_id != 0 {
        let updated_count = referral_utils::update_ancestors_total_staked(
            &storage_accounts,
            user_parent_id,
            REFERRAL_UPDATE_LEVELS,
            ctx.program_id,
            amount as i64,  // positive value means accumulate
        )?;

        msg!("Updated {} ancestors' total_staked", updated_count);
    } else {
        msg!("User is root node, no ancestors to update");
    }

    msg!("✅ Stake created successfully: user={}, amount={}, period={}, order_index={}",
        user.key(), amount, period_type, order_index);

    emit!(StakeCreatedEvent {
        user: user.key(),
        referral_id: wallet_mapping.referral_id,
        order_index: order_index as u8,
        amount,
        period_type,
        initial_daily_rate,
        start_time: current_time,
        end_time,
        global_total_staked: global_state.total_staked,
        daily_deposited: global_state.daily_deposited,
        daily_deposit_cap: global_state.daily_deposit_cap,
        user_stake_cap: user_cap,
    });

    Ok(())
}

/// Find an empty order slot in the orders array
fn find_empty_order_slot(orders: &[StakeOrder; MAX_STAKES_PER_USER]) -> Result<usize> {
    for (index, order) in orders.iter().enumerate() {
        // VEL-06: 使用 ORDER_STATUS_EMPTY 判断空槽位
        if order.status == ORDER_STATUS_EMPTY || order.status == ORDER_STATUS_COMPLETED {
            return Ok(index);
        }
    }
    Err(StakeError::MaxStakesReached.into())
}

/// Unstake: redeem principal and remaining interest after period ends
pub fn handler_unstake(ctx: Context<Unstake>, order_index: u8) -> Result<()> {
    let user = &ctx.accounts.user;
    let user_stake_account = &mut ctx.accounts.user_stake_account;
    let global_state = &mut ctx.accounts.global_state;
    let wallet_mapping = &ctx.accounts.wallet_mapping;
    let locked_vault = &ctx.accounts.locked_vault;

    // ========== 0. Verify 9 storage PDA addresses ==========
    verify_storage_pdas!(ctx, global_state);

    // ========== 1. Verify LockedVault PDA ==========
    let token_mint = global_state.stake_token_mint;
    let (expected_locked_vault, _bump) = Pubkey::find_program_address(
        &[LOCKED_VAULT_SEED, token_mint.as_ref()],
        ctx.program_id,
    );
    require!(
        locked_vault.key() == expected_locked_vault,
        StakeError::InvalidLockedVault
    );

    // ========== 2. Verify order index ==========
    require!(
        (order_index as usize) < MAX_STAKES_PER_USER,
        StakeError::InvalidOrderIndex
    );

    // ========== 3. Verify order status ==========
    let current_time = Clock::get()?.unix_timestamp;
    {
        let order = &user_stake_account.orders[order_index as usize];
        require!(order.status == ORDER_STATUS_ACTIVE, StakeError::OrderNotActive);
        // VEL-06: 双重保险，防止任何零值槽位通过
        require!(order.amount > 0, StakeError::OrderNotActive);

        // ========== 4. Verify staking period has ended ==========
        require!(current_time >= order.end_time, StakeError::PeriodNotEnded);
    }

    // ========== 5. Calculate latest interest first (update accumulated_interest in place) ==========
    {
        let order = &mut user_stake_account.orders[order_index as usize];
        // Interest is only calculated up to end_time; no more interest accrues after expiry
        let effective_time = current_time.min(order.end_time);
        let hours_passed = ((effective_time - order.last_interest_time) / SECONDS_PER_HOUR as i64) as u64;
        if hours_passed > 0 {
            // CI-05/06: Multiply first then divide, avoiding integer division precision loss from hourly_rate = daily_rate / 24
            // Consistent calculation method with calc_pending_interest
            let new_interest = u64::try_from(
                (order.amount as u128)
                    .checked_mul(hours_passed as u128)
                    .ok_or(StakeError::ArithmeticOverflow)?
                    .checked_mul(order.initial_daily_rate as u128)
                    .ok_or(StakeError::ArithmeticOverflow)?
                    .checked_div(RATE_BASIS_POINTS as u128 * 24)
                    .ok_or(StakeError::ArithmeticOverflow)?
            ).map_err(|_| StakeError::ArithmeticOverflow)?;

            order.accumulated_interest = order.accumulated_interest
                .checked_add(new_interest)
                .ok_or(StakeError::ArithmeticOverflow)?;
            order.last_interest_time += (hours_passed * SECONDS_PER_HOUR) as i64;
        }
    }

    // ========== 6. Read order data (borrow separation) ==========
    let principal;
    let base_interest;
    {
        let order = &user_stake_account.orders[order_index as usize];
        principal = order.amount;
        base_interest = order.accumulated_interest;
    }

    // ========== 7. Calculate NFT boost (separated from base interest for future independent handling) ==========
    // base_interest:  base interest generated by staking duration * daily rate
    // boost_interest: extra bonus interest from NFT nodes (proportionally calculated based on base_interest)
    // total_interest = base_interest + boost_interest (total unchanged for now; in the future, each can be taxed/split separately)
    let nft_boost_bps = get_nft_boost_bps(
        &ctx.accounts.user.key(),
        &ctx.accounts.user_state,
        &ctx.accounts.nft_binding_state,
        &ctx.accounts.user_nft_account,
        ctx.program_id,
    )?;

    let boost_interest = if nft_boost_bps > 0 {
        u64::try_from(
            (base_interest as u128)
                .checked_mul(nft_boost_bps as u128)
                .ok_or(StakeError::ArithmeticOverflow)?
                .checked_div(BASIS_POINTS as u128)
                .ok_or(StakeError::ArithmeticOverflow)?
        ).map_err(|_| StakeError::ArithmeticOverflow)?
    } else {
        0
    };

    let total_interest = base_interest
        .checked_add(boost_interest)
        .ok_or(StakeError::ArithmeticOverflow)?;

    msg!("Unstake: principal={}, base_interest={}, boost_interest={}, total_interest={}, boost_bps={}",
        principal, base_interest, boost_interest, total_interest, nft_boost_bps);

    // ========== 8. Calculate tax amount and user's net interest ==========
    // Currently: tax is uniformly calculated based on total_interest (base + boost)
    // Future: different tax rates may apply to base_interest and boost_interest separately
    let tax_amount: u64;
    let final_interest_to_user: u64;

    if total_interest > 0 {
        tax_amount = u64::try_from(
            (total_interest as u128)
                .checked_mul(INTEREST_TAX_RATE as u128)
                .ok_or(StakeError::ArithmeticOverflow)?
                .checked_div(BASIS_POINTS as u128)
                .ok_or(StakeError::ArithmeticOverflow)?
        ).map_err(|_| StakeError::ArithmeticOverflow)?;

        final_interest_to_user = total_interest
            .checked_sub(tax_amount)
            .ok_or(StakeError::ArithmeticOverflow)?;
    } else {
        tax_amount = 0;
        final_interest_to_user = 0;
    }

    msg!("Interest: tax={}, user_interest={}", tax_amount, final_interest_to_user);

    // ========== 9. Transfer tax amount to dead address ==========
    let vault_bump = locked_vault.bump;
    let locked_vault_key = locked_vault.key();
    let _ = locked_vault_key; // suppress unused warning

    if tax_amount > 0 {
        let signer_seeds: &[&[&[u8]]] = &[&[LOCKED_VAULT_SEED, token_mint.as_ref(), &[vault_bump]]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.dead_address_token_account.to_account_info(),
            authority: ctx.accounts.locked_vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer(cpi_ctx, tax_amount)?;
        msg!("Transferred tax {} to dead address", tax_amount);
    }

    // ========== 10. Transfer principal + net interest to user ==========
    let total_to_user = principal
        .checked_add(final_interest_to_user)
        .ok_or(StakeError::ArithmeticOverflow)?;

    {
        let signer_seeds: &[&[&[u8]]] = &[&[LOCKED_VAULT_SEED, token_mint.as_ref(), &[vault_bump]]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.locked_vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer(cpi_ctx, total_to_user)?;
    }

    msg!("Transferred {} (principal={} + interest={}) to user", total_to_user, principal, final_interest_to_user);

    // ========== 11. Update order status ==========
    {
        let order = &mut user_stake_account.orders[order_index as usize];
        order.status = ORDER_STATUS_COMPLETED;
        if total_interest > 0 {
            order.claimed_interest = order.claimed_interest
                .checked_add(total_interest)
                .ok_or(StakeError::ArithmeticOverflow)?;
        }
        order.accumulated_interest = 0;
    }

    // ========== 12. Update user account statistics ==========
    user_stake_account.active_count = user_stake_account.active_count
        .checked_sub(1)
        .ok_or(StakeError::ArithmeticOverflow)?;
    user_stake_account.total_principal = user_stake_account.total_principal
        .checked_sub(principal)
        .ok_or(StakeError::ArithmeticOverflow)?;
    if total_interest > 0 {
        user_stake_account.total_claimed_interest = user_stake_account.total_claimed_interest
            .checked_add(total_interest)
            .ok_or(StakeError::ArithmeticOverflow)?;
    }

    msg!("Updated user stats: active_count={}, total_principal={}", user_stake_account.active_count, user_stake_account.total_principal);

    // ========== 13. Update global state ==========
    global_state.total_staked = global_state.total_staked
        .checked_sub(principal)
        .ok_or(StakeError::ArithmeticOverflow)?;
    if total_interest > 0 {
        global_state.total_interest_paid = global_state.total_interest_paid
            .checked_add(total_interest)
            .ok_or(StakeError::ArithmeticOverflow)?;
    }

    msg!("Updated global state: total_staked={}", global_state.total_staked);

    // ========== 14. Update referral system: subtract total_staked upward 50 levels ==========
    let storage_accounts = [
        &ctx.accounts.storage_1,
        &ctx.accounts.storage_2,
        &ctx.accounts.storage_3,
        &ctx.accounts.storage_4,
        &ctx.accounts.storage_5,
        &ctx.accounts.storage_6,
        &ctx.accounts.storage_7,
        &ctx.accounts.storage_8,
        &ctx.accounts.storage_9,
    ];

    let user_referral_id = wallet_mapping.referral_id;
    let (pda_index, slot_index) = ReferralStorage::decode_and_validate_id(user_referral_id)?;

    // Update own self_staked and read parent_id
    let user_parent_id = {
        let storage_account = storage_accounts[(pda_index - 1) as usize];
        let mut storage_data = storage_account.try_borrow_mut_data()?;
        let mut user_referral_data = zero_copy_storage::read_record(&storage_data, slot_index)?;
        let parent_id = user_referral_data.parent_id;
        user_referral_data.self_staked = user_referral_data.self_staked.saturating_sub(principal);
        zero_copy_storage::write_record(&mut storage_data, slot_index, &user_referral_data)?;
        parent_id
    };

    msg!("User referral_id: {}, parent_id: {}, self_staked -= {}", user_referral_id, user_parent_id, principal);

    if user_parent_id != 0 {
        let updated_count = referral_utils::update_ancestors_total_staked(
            &storage_accounts,
            user_parent_id,
            REFERRAL_UPDATE_LEVELS,
            ctx.program_id,
            -(principal as i64),  // negative value means subtract
        )?;
        msg!("Updated {} ancestors' total_staked (decremented)", updated_count);
    } else {
        msg!("User is root node, no ancestors to update");
    }

    // ========== 15. Community reward distribution (based on base_interest, excluding NFT boost) ==========
    let storage_account_infos: [&AccountInfo; 9] = [
        &ctx.accounts.storage_1,
        &ctx.accounts.storage_2,
        &ctx.accounts.storage_3,
        &ctx.accounts.storage_4,
        &ctx.accounts.storage_5,
        &ctx.accounts.storage_6,
        &ctx.accounts.storage_7,
        &ctx.accounts.storage_8,
        &ctx.accounts.storage_9,
    ];

    let distributed = if base_interest > 0 {
        let dist = community_reward::distribute_community_reward(
            &storage_account_infos,
            user_referral_id,
            base_interest,
            ctx.program_id,
            global_state,
            current_time,
        )?;
        msg!("Community reward distributed: {} (base_interest={})", dist, base_interest);
        dist
    } else {
        // Even if no interest, still trigger week refresh
        community_reward::maybe_refresh_week(global_state, current_time, &storage_account_infos)?;
        0u64
    };

    // ========== 16. Accumulate total_output (interest + community rewards) ==========
    // CI-04: Accounting note: total_output = total actual vault expenditure
    //   total_interest = user_receive + tax_amount (including NFT boost, pre-tax total interest)
    //   distributed = community reward distribution amount (calculated based on base_interest)
    //   The sum of both is the total vault expenditure for this operation, driving the production reduction mechanism
    {
        let total_output_amount = total_interest
            .checked_add(distributed)
            .ok_or(StakeError::ArithmeticOverflow)?;
        global_state.total_output = global_state.total_output
            .checked_add(total_output_amount)
            .ok_or(StakeError::ArithmeticOverflow)?;
        global_state.reduction_count = calc_reduction_count(global_state.total_output);
        msg!("Updated total_output: {}, reduction_count: {}", global_state.total_output, global_state.reduction_count);
    }

    msg!("✅ Unstake completed: user={}, principal={}, base_interest={}, boost_interest={}, interest_to_user={}, tax={}, order_index={}",
        user.key(), principal, base_interest, boost_interest, final_interest_to_user, tax_amount, order_index);

    let nft_mint = ctx.accounts.user_state
        .as_ref()
        .map(|s| s.bound_nft_mint)
        .unwrap_or_default();

    emit!(UnstakedEvent {
        user: user.key(),
        referral_id: wallet_mapping.referral_id,
        order_index,
        principal,
        base_interest,
        boost_interest,
        total_interest,
        tax_amount,
        interest_to_user: final_interest_to_user,
        total_to_user,
        nft_boost_bps,
        nft_mint,
        global_total_staked: global_state.total_staked,
        global_total_interest_paid: global_state.total_interest_paid,
        timestamp: current_time,
    });

    Ok(())
}

/// Pure function to calculate pending interest (does not write on-chain state)
///
/// Parameters:
/// - order: order reference (immutable)
/// - current_time: current timestamp
/// - nft_boost_bps: NFT boost ratio (basis points, 0 = no boost)
///
/// Returns PendingInterestResult
fn calc_pending_interest(
    order: &StakeOrder,
    current_time: i64,
    nft_boost_bps: u64,
) -> Result<PendingInterestResult> {
    // VEL-06: Return all zeros if the order is not active
    if order.status != ORDER_STATUS_ACTIVE {
        return Ok(PendingInterestResult {
            base_interest: 0,
            boost_interest: 0,
            total_interest: 0,
            after_tax: 0,
            tax_amount: 0,
        });
    }

    // Interest is only calculated up to end_time
    let effective_time = current_time.min(order.end_time);
    let hours_passed = ((effective_time - order.last_interest_time) / SECONDS_PER_HOUR as i64) as u64;

    let new_interest = if hours_passed > 0 {
        u64::try_from(
            (order.amount as u128)
                .checked_mul(hours_passed as u128)
                .ok_or(StakeError::ArithmeticOverflow)?
                .checked_mul(order.initial_daily_rate as u128)
                .ok_or(StakeError::ArithmeticOverflow)?
                .checked_div(RATE_BASIS_POINTS as u128 * 24)
                .ok_or(StakeError::ArithmeticOverflow)?
        ).map_err(|_| StakeError::ArithmeticOverflow)?
    } else {
        0
    };

    let base_interest = order.accumulated_interest
        .checked_add(new_interest)
        .ok_or(StakeError::ArithmeticOverflow)?;

    let boost_interest = if nft_boost_bps > 0 {
        u64::try_from(
            (base_interest as u128)
                .checked_mul(nft_boost_bps as u128)
                .ok_or(StakeError::ArithmeticOverflow)?
                .checked_div(BASIS_POINTS as u128)
                .ok_or(StakeError::ArithmeticOverflow)?
        ).map_err(|_| StakeError::ArithmeticOverflow)?
    } else {
        0
    };

    let total_interest = base_interest
        .checked_add(boost_interest)
        .ok_or(StakeError::ArithmeticOverflow)?;

    let tax_amount = u64::try_from(
        (total_interest as u128)
            .checked_mul(INTEREST_TAX_RATE as u128)
            .ok_or(StakeError::ArithmeticOverflow)?
            .checked_div(BASIS_POINTS as u128)
            .ok_or(StakeError::ArithmeticOverflow)?
    ).map_err(|_| StakeError::ArithmeticOverflow)?;

    let after_tax = total_interest
        .checked_sub(tax_amount)
        .ok_or(StakeError::ArithmeticOverflow)?;

    Ok(PendingInterestResult {
        base_interest,
        boost_interest,
        total_interest,
        after_tax,
        tax_amount,
    })
}

/// Get NFT boost basis points (read from user_state and nft_binding_state)
/// Includes runtime validation to prevent using another user's NFT boost
fn get_nft_boost_bps(
    user_key: &Pubkey,
    user_state: &Option<Box<Account<UserState>>>,
    nft_binding_state: &Option<Box<Account<NftBindingState>>>,
    user_nft_account: &Option<Box<Account<TokenAccount>>>,
    program_id: &Pubkey,
) -> Result<u64> {
    let boost_bps = if let Some(user_state_acc) = user_state {

        // Verify user_state PDA belongs to current signer
        let (expected_user_state, _) = Pubkey::find_program_address(
            &[USER_STATE_SEED, user_key.as_ref()],
            program_id,
        );
        require!(
            user_state_acc.key() == expected_user_state,
            StakeError::Unauthorized
        );

        if user_state_acc.bound_nft_mint != Pubkey::default() {
            if let Some(nft_binding) = nft_binding_state {
                // Verify nft_binding mint matches user_state (existing check)
                require!(
                    nft_binding.nft_mint == user_state_acc.bound_nft_mint,
                    StakeError::NftBindingMismatch
                );

                // Verify nft_binding_state PDA correctness
                let (expected_nft_binding, _) = Pubkey::find_program_address(
                    &[NFT_BINDING_SEED, nft_binding.nft_mint.as_ref()],
                    program_id,
                );
                require!(
                    nft_binding.key() == expected_nft_binding,
                    StakeError::NftBindingMismatch
                );

                // Verify nft_binding_state.owner is current signer
                require!(
                    nft_binding.owner == *user_key,
                    StakeError::Unauthorized
                );

                // Verify user actually holds the NFT via token account
                if let Some(nft_account) = user_nft_account {
                    require!(
                        nft_account.owner == *user_key,
                        StakeError::Unauthorized
                    );
                    require!(
                        nft_account.mint == nft_binding.nft_mint,
                        StakeError::NftBindingMismatch
                    );
                    require!(
                        nft_account.amount > 0,
                        StakeError::Unauthorized
                    );
                } else {
                    // No NFT token account provided, cannot confirm holding, deny boost
                    return Ok(0);
                }

                match nft_binding.node_type {
                    NODE_TYPE_DIAMOND => DIAMOND_NODE_BOOST,
                    NODE_TYPE_GOLD => GOLD_NODE_BOOST,
                    _ => 0,
                }
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };
    Ok(boost_bps)
}

/// Claim interest: claim accumulated interest for a specified order (including NFT boost and tax)
pub fn handler_claim_interest(ctx: Context<ClaimInterest>, order_index: u8) -> Result<()> {
    let user = &ctx.accounts.user;
    let user_stake_account = &mut ctx.accounts.user_stake_account;
    let global_state = &mut ctx.accounts.global_state;
    let locked_vault = &ctx.accounts.locked_vault;

    // ========== 1. Verify LockedVault PDA ==========
    let token_mint = global_state.stake_token_mint;
    let (expected_locked_vault, _bump) = Pubkey::find_program_address(
        &[LOCKED_VAULT_SEED, token_mint.as_ref()],
        ctx.program_id,
    );
    require!(
        locked_vault.key() == expected_locked_vault,
        StakeError::InvalidLockedVault
    );

    // ========== 2. Verify order index ==========
    require!(
        (order_index as usize) < MAX_STAKES_PER_USER,
        StakeError::InvalidOrderIndex
    );

    // ========== 3. Verify order status ==========
    {
        let order = &user_stake_account.orders[order_index as usize];
        require!(order.status == ORDER_STATUS_ACTIVE, StakeError::OrderNotActive);
        // VEL-06: 双重保险，防止任何零值槽位通过
        require!(order.amount > 0, StakeError::OrderNotActive);
    }

    // ========== 4. Accumulate latest interest (update accumulated_interest in place) ==========
    let current_time = Clock::get()?.unix_timestamp;

    // CI-07: Minimum claim interval check (at least 1 SECONDS_PER_HOUR)
    {
        let order = &user_stake_account.orders[order_index as usize];
        require!(
            current_time - order.last_interest_time >= SECONDS_PER_HOUR as i64,
            StakeError::ClaimTooFrequent
        );
    }

    {
        let order = &mut user_stake_account.orders[order_index as usize];
        // Interest is only calculated up to end_time
        let effective_time = current_time.min(order.end_time);
        let hours_passed = ((effective_time - order.last_interest_time) / SECONDS_PER_HOUR as i64) as u64;
        if hours_passed > 0 {
            // CI-05/06: Multiply first then divide, avoiding integer division precision loss from hourly_rate = daily_rate / 24
            // Consistent calculation method with calc_pending_interest
            let new_interest = u64::try_from(
                (order.amount as u128)
                    .checked_mul(hours_passed as u128)
                    .ok_or(StakeError::ArithmeticOverflow)?
                    .checked_mul(order.initial_daily_rate as u128)
                    .ok_or(StakeError::ArithmeticOverflow)?
                    .checked_div(RATE_BASIS_POINTS as u128 * 24)
                    .ok_or(StakeError::ArithmeticOverflow)?
            ).map_err(|_| StakeError::ArithmeticOverflow)?;

            order.accumulated_interest = order.accumulated_interest
                .checked_add(new_interest)
                .ok_or(StakeError::ArithmeticOverflow)?;
            order.last_interest_time += (hours_passed * SECONDS_PER_HOUR) as i64;
        }
    }

    // ========== 5. Verify there is interest to claim ==========
    let base_interest;
    {
        let order = &user_stake_account.orders[order_index as usize];
        base_interest = order.accumulated_interest;
    }
    require!(base_interest > 0, StakeError::NoInterestToClaim);

    // ========== 6. Calculate NFT boost (separated from base interest for future independent handling) ==========
    // base_interest:  base interest generated by staking duration * daily rate
    // boost_interest: extra bonus interest from NFT nodes (proportionally calculated based on base_interest)
    // total_interest = base_interest + boost_interest (total unchanged for now; in the future, each can be taxed/split separately)
    let nft_boost_bps = get_nft_boost_bps(
        &ctx.accounts.user.key(),
        &ctx.accounts.user_state,
        &ctx.accounts.nft_binding_state,
        &ctx.accounts.user_nft_account,
        ctx.program_id,
    )?;

    let boost_interest = if nft_boost_bps > 0 {
        u64::try_from(
            (base_interest as u128)
                .checked_mul(nft_boost_bps as u128)
                .ok_or(StakeError::ArithmeticOverflow)?
                .checked_div(BASIS_POINTS as u128)
                .ok_or(StakeError::ArithmeticOverflow)?
        ).map_err(|_| StakeError::ArithmeticOverflow)?
    } else {
        0
    };

    let total_interest = base_interest
        .checked_add(boost_interest)
        .ok_or(StakeError::ArithmeticOverflow)?;

    msg!("ClaimInterest: base={}, boost={}, total={}, boost_bps={}",
        base_interest, boost_interest, total_interest, nft_boost_bps);

    // ========== 7. Calculate tax amount and user's net amount ==========
    // Currently: tax is uniformly calculated based on total_interest (base + boost)
    // Future: different tax rates may apply to base_interest and boost_interest separately
    let tax_amount = u64::try_from(
        (total_interest as u128)
            .checked_mul(INTEREST_TAX_RATE as u128)
            .ok_or(StakeError::ArithmeticOverflow)?
            .checked_div(BASIS_POINTS as u128)
            .ok_or(StakeError::ArithmeticOverflow)?
    ).map_err(|_| StakeError::ArithmeticOverflow)?;

    let user_receive = total_interest
        .checked_sub(tax_amount)
        .ok_or(StakeError::ArithmeticOverflow)?;

    msg!("Interest: tax={}, user_receive={}", tax_amount, user_receive);

    // ========== 8. Transfer tax amount to dead address ==========
    let vault_bump = locked_vault.bump;

    if tax_amount > 0 {
        let signer_seeds: &[&[&[u8]]] = &[&[LOCKED_VAULT_SEED, token_mint.as_ref(), &[vault_bump]]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.dead_address_token_account.to_account_info(),
            authority: ctx.accounts.locked_vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer(cpi_ctx, tax_amount)?;
        msg!("Transferred tax {} to dead address", tax_amount);
    }

    // ========== 9. Transfer user's net interest to user account ==========
    if user_receive > 0 {
        let signer_seeds: &[&[&[u8]]] = &[&[LOCKED_VAULT_SEED, token_mint.as_ref(), &[vault_bump]]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.locked_vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer(cpi_ctx, user_receive)?;
    }

    msg!("Transferred {} interest to user", user_receive);

    // ========== 9.5 Community reward distribution (based on base_interest, excluding NFT boost) ==========
    let distributed = if base_interest > 0 {
        // Verify 9 storage PDA addresses
        verify_storage_pdas!(ctx, global_state);

        let storage_accounts: [&AccountInfo; 9] = [
            &ctx.accounts.storage_1,
            &ctx.accounts.storage_2,
            &ctx.accounts.storage_3,
            &ctx.accounts.storage_4,
            &ctx.accounts.storage_5,
            &ctx.accounts.storage_6,
            &ctx.accounts.storage_7,
            &ctx.accounts.storage_8,
            &ctx.accounts.storage_9,
        ];

        let user_referral_id = ctx.accounts.wallet_mapping.referral_id;
        let dist = community_reward::distribute_community_reward(
            &storage_accounts,
            user_referral_id,
            base_interest,
            ctx.program_id,
            global_state,
            Clock::get()?.unix_timestamp,
        )?;
        msg!("Community reward distributed: {} (base_interest={})", dist, base_interest);
        dist
    } else {
        0u64
    };

    // ========== 10. Update order status ==========
    {
        let order = &mut user_stake_account.orders[order_index as usize];
        order.claimed_interest = order.claimed_interest
            .checked_add(total_interest)
            .ok_or(StakeError::ArithmeticOverflow)?;
        order.accumulated_interest = 0;
    }

    // ========== 11. Update user account statistics ==========
    user_stake_account.total_claimed_interest = user_stake_account.total_claimed_interest
        .checked_add(total_interest)
        .ok_or(StakeError::ArithmeticOverflow)?;

    // ========== 12. Update global state ==========
    global_state.total_interest_paid = global_state.total_interest_paid
        .checked_add(total_interest)
        .ok_or(StakeError::ArithmeticOverflow)?;

    // ========== 12.5 Accumulate total_output (interest + community rewards) ==========
    // CI-04: Accounting note: total_output = total actual vault expenditure
    //   total_interest = user_receive + tax_amount (including NFT boost, pre-tax total interest)
    //   distributed = community reward distribution amount (calculated based on base_interest)
    //   The sum of both is the total vault expenditure for this operation, driving the production reduction mechanism
    {
        let total_output_amount = total_interest
            .checked_add(distributed)
            .ok_or(StakeError::ArithmeticOverflow)?;
        global_state.total_output = global_state.total_output
            .checked_add(total_output_amount)
            .ok_or(StakeError::ArithmeticOverflow)?;
        global_state.reduction_count = calc_reduction_count(global_state.total_output);
        msg!("Updated total_output: {}, reduction_count: {}", global_state.total_output, global_state.reduction_count);
    }

    msg!("✅ ClaimInterest completed: user={}, base_interest={}, boost={}, tax={}, user_receive={}, order_index={}",
        user.key(), base_interest, boost_interest, tax_amount, user_receive, order_index);

    let nft_mint = ctx.accounts.user_state
        .as_ref()
        .map(|s| s.bound_nft_mint)
        .unwrap_or_default();

    emit!(InterestClaimedEvent {
        user: user.key(),
        order_index,
        base_interest,
        boost_interest,
        total_interest,
        tax_amount,
        user_receive,
        nft_boost_bps,
        nft_mint,
        global_total_interest_paid: global_state.total_interest_paid,
        timestamp: current_time,
    });

    Ok(())
}

/// Query pending interest: query the expected pending interest for a specified order (read-only, does not modify on-chain state)
pub fn handler_query_pending_interest(
    ctx: Context<QueryPendingInterest>,
    order_index: u8,
) -> Result<PendingInterestResult> {
    // ========== 1. Verify order index ==========
    require!(
        (order_index as usize) < MAX_STAKES_PER_USER,
        StakeError::InvalidOrderIndex
    );

    let current_time = Clock::get()?.unix_timestamp;
    let order = &ctx.accounts.user_stake_account.orders[order_index as usize];

    // ========== 2. Get NFT boost ==========
    let nft_boost_bps = get_nft_boost_bps(
        &ctx.accounts.user.key(),
        &ctx.accounts.user_state,
        &ctx.accounts.nft_binding_state,
        &ctx.accounts.user_nft_account,
        ctx.program_id,
    )?;

    // ========== 3. Calculate pending interest ==========
    let result = calc_pending_interest(order, current_time, nft_boost_bps)?;

    msg!("QueryPendingInterest: order_index={}, base={}, boost={}, total={}, after_tax={}, tax={}",
        order_index,
        result.base_interest,
        result.boost_interest,
        result.total_interest,
        result.after_tax,
        result.tax_amount,
    );

    Ok(result)
}

/// Query community status: query the user's community status (read-only, use .view())
pub fn handler_query_community_status(
    ctx: Context<QueryCommunityStatus>,
) -> Result<CommunityStatusResult> {
    let wallet_mapping = &ctx.accounts.wallet_mapping;
    let referral_id = wallet_mapping.referral_id;

    let (pda_index, slot_index) = ReferralStorage::decode_and_validate_id(referral_id)?;

    let storage_accounts: [&AccountInfo; 9] = [
        &ctx.accounts.storage_1,
        &ctx.accounts.storage_2,
        &ctx.accounts.storage_3,
        &ctx.accounts.storage_4,
        &ctx.accounts.storage_5,
        &ctx.accounts.storage_6,
        &ctx.accounts.storage_7,
        &ctx.accounts.storage_8,
        &ctx.accounts.storage_9,
    ];

    let storage_account = storage_accounts[(pda_index - 1) as usize];
    let storage_data = storage_account.try_borrow_data()?;
    let referral = zero_copy_storage::read_record(&storage_data, slot_index)?;

    let level = community_reward::determine_level(referral.self_staked, referral.total_staked);
    let total_community_profit = referral.direct_reward_profit.saturating_add(referral.team_reward_profit);

    msg!("QueryCommunityStatus: referral_id={}, parent_id={}, self_staked={}, total_staked={}, direct={}, team={}, total={}, level={}",
        referral_id, referral.parent_id, referral.self_staked, referral.total_staked,
        referral.direct_reward_profit, referral.team_reward_profit, total_community_profit, level);

    Ok(CommunityStatusResult {
        referral_id,
        parent_id: referral.parent_id,
        self_staked: referral.self_staked,
        total_staked: referral.total_staked,
        direct_reward_profit: referral.direct_reward_profit,
        team_reward_profit: referral.team_reward_profit,
        total_community_profit,
        level,
    })
}

/// Claim community profit: claim community rewards (transfer from locked_vault to user)
pub fn handler_claim_community_profit(
    ctx: Context<ClaimCommunityProfit>,
) -> Result<()> {
    let wallet_mapping = &ctx.accounts.wallet_mapping;
    let locked_vault = &ctx.accounts.locked_vault;
    let global_state = &ctx.accounts.global_state;

    // ========== 1. Verify 9 storage PDA addresses ==========
    verify_storage_pdas!(ctx, global_state);

    // ========== 2. Verify LockedVault PDA ==========
    let token_mint = global_state.stake_token_mint;
    let (expected_locked_vault, _bump) = Pubkey::find_program_address(
        &[LOCKED_VAULT_SEED, token_mint.as_ref()],
        ctx.program_id,
    );
    require!(
        locked_vault.key() == expected_locked_vault,
        StakeError::InvalidLockedVault
    );

    // ========== 3. Read community_profit ==========
    let referral_id = wallet_mapping.referral_id;
    let (pda_index, slot_index) = ReferralStorage::decode_and_validate_id(referral_id)?;

    let storage_accounts: [&AccountInfo; 9] = [
        &ctx.accounts.storage_1,
        &ctx.accounts.storage_2,
        &ctx.accounts.storage_3,
        &ctx.accounts.storage_4,
        &ctx.accounts.storage_5,
        &ctx.accounts.storage_6,
        &ctx.accounts.storage_7,
        &ctx.accounts.storage_8,
        &ctx.accounts.storage_9,
    ];

    let storage_account = storage_accounts[(pda_index - 1) as usize];

    // ========== 3. Read two profit fields and calculate total ==========
    let (direct_profit, team_profit, total_profit) = {
        let storage_data = storage_account.try_borrow_data()?;
        let referral = zero_copy_storage::read_record(&storage_data, slot_index)?;
        let total = referral.direct_reward_profit.saturating_add(referral.team_reward_profit);
        (referral.direct_reward_profit, referral.team_reward_profit, total)
    };

    require!(total_profit > 0, StakeError::NoInterestToClaim);

    msg!("ClaimCommunityProfit: referral_id={}, direct={}, team={}, total={}",
        referral_id, direct_profit, team_profit, total_profit);

    // ========== 4. Calculate tax ==========
    let tax_amount = u64::try_from(
        (total_profit as u128)
            .checked_mul(INTEREST_TAX_RATE as u128)
            .ok_or(StakeError::ArithmeticOverflow)?
            .checked_div(BASIS_POINTS as u128)
            .ok_or(StakeError::ArithmeticOverflow)?
    ).map_err(|_| StakeError::ArithmeticOverflow)?;

    let user_receive = total_profit
        .checked_sub(tax_amount)
        .ok_or(StakeError::ArithmeticOverflow)?;

    msg!("CommunityProfit tax: tax={}, user_receive={}", tax_amount, user_receive);

    // ========== 5. Transfer tax to dead address ==========
    let vault_bump = locked_vault.bump;
    if tax_amount > 0 {
        let signer_seeds: &[&[&[u8]]] = &[&[LOCKED_VAULT_SEED, token_mint.as_ref(), &[vault_bump]]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.dead_address_token_account.to_account_info(),
            authority: ctx.accounts.locked_vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer(cpi_ctx, tax_amount)?;
        msg!("Transferred tax {} to dead address", tax_amount);
    }

    // ========== 6. Transfer remaining to user ==========
    {
        let signer_seeds: &[&[&[u8]]] = &[&[LOCKED_VAULT_SEED, token_mint.as_ref(), &[vault_bump]]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.locked_vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer(cpi_ctx, user_receive)?;
    }

    msg!("Transferred {} community profit to user (tax: {})", user_receive, tax_amount);

    // ========== 7. Reset both profit fields to zero ==========
    {
        let mut storage_data = storage_account.try_borrow_mut_data()?;
        let mut referral = zero_copy_storage::read_record(&storage_data, slot_index)?;
        referral.direct_reward_profit = 0;
        referral.team_reward_profit = 0;
        zero_copy_storage::write_record(&mut storage_data, slot_index, &referral)?;
    }

    msg!("✅ ClaimCommunityProfit completed: user={}, amount={}", ctx.accounts.user.key(), total_profit);

    emit!(CommunityProfitClaimedEvent {
        user: ctx.accounts.user.key(),
        referral_id,
        direct_reward_amount: direct_profit,
        team_reward_amount: team_profit,
        total_amount: total_profit,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

/// Query the current daily rates for the four lock-up tiers
pub fn handler_query_current_rates(
    ctx: Context<QueryCurrentRates>,
) -> Result<CurrentRatesResult> {
    let global_state = &ctx.accounts.global_state;
    let reduction_count = calc_reduction_count(global_state.total_output);
    let (rate_7d, rate_30d, rate_90d, rate_365d) = get_current_rates(reduction_count);

    msg!("QueryCurrentRates: total_output={}, reduction_count={}, rate_7d={}, rate_30d={}, rate_90d={}, rate_365d={}",
        global_state.total_output, reduction_count, rate_7d, rate_30d, rate_90d, rate_365d);

    Ok(CurrentRatesResult {
        total_output: global_state.total_output,
        reduction_count,
        rate_7d,
        rate_30d,
        rate_90d,
        rate_365d,
    })
}
