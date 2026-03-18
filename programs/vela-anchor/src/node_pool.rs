use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
use crate::{
    constants::*,
    structs::*,
    errors::NodePoolError,
    events::NodePoolRewardClaimed,
    community_reward::{maybe_refresh_week, calculate_week_number},
};

/// Claim node pool reward
pub fn handler_claim_node_pool_reward(
    ctx: Context<crate::contexts::ClaimNodePoolReward>,
) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;
    let nft_binding = &mut ctx.accounts.nft_binding_state;
    let locked_vault = &ctx.accounts.locked_vault;
    let current_time = Clock::get()?.unix_timestamp;

    // ========== 0. Validate referral_storage_1 address ==========
    require_keys_eq!(
        ctx.accounts.referral_storage_1.key(),
        global_state.storage_pdas[0],
        crate::errors::ReferralError::InvalidStoragePDA
    );

    // ========== 1. Cross-week detection & refresh ==========
    // Static assertion: ensure ROOT_REFERRAL_ID maps to pda_index=1, i.e. storage_accounts[0]
    debug_assert_eq!(ReferralStorage::decode_id(ROOT_REFERRAL_ID).0, 1);
    let storage_accounts: [&AccountInfo; 1] = [
        &ctx.accounts.referral_storage_1,
    ];
    // For claim_node_pool_reward, only pass storage_1 (which contains root)
    // Build a 9-element slice to satisfy maybe_refresh_week's requirement
    // But maybe_refresh_week only accesses storage_1 (where root resides), so passing 1 is sufficient
    // We need to pass storage_accounts to maybe_refresh_week, which expects &[&AccountInfo]
    // add_community_profit resolves ROOT_REFERRAL_ID to pda_index=1, slot_index=0
    // So only storage_accounts[0] (i.e. the PDA at index=1) is needed
    maybe_refresh_week(global_state, current_time, &storage_accounts)?;

    // ========== 2. Check current_week_number > 0 ==========
    require!(
        global_state.current_week_number > 0,
        NodePoolError::NoPreviousWeekData
    );

    let previous_week = global_state.current_week_number - 1;

    // ========== 3. Check if last week's reward has already been claimed ==========
    require!(
        nft_binding.last_pool_claim_week != previous_week,
        NodePoolError::AlreadyClaimedThisWeek
    );

    // ========== 4. Determine node type and calculate amount ==========
    let node_type = nft_binding.node_type;
    let amount = match node_type {
        NODE_TYPE_DIAMOND => {
            if DIAMOND_POOL_SHARES == 0 { 0 } else {
                global_state.diamond_pool_previous / DIAMOND_POOL_SHARES
            }
        }
        NODE_TYPE_GOLD => {
            if GOLD_POOL_SHARES == 0 { 0 } else {
                global_state.gold_pool_previous / GOLD_POOL_SHARES
            }
        }
        _ => 0,
    };

    require!(amount > 0, NodePoolError::NoPoolRewards);

    // ========== 5. Balance pre-check ==========
    require!(
        ctx.accounts.vault_token_account.amount >= amount,
        NodePoolError::InsufficientVaultBalance
    );

    // ========== 6. Transfer from Vault to user ==========
    let token_mint = locked_vault.token_mint;
    let vault_bump = locked_vault.bump;
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
    token::transfer(cpi_ctx, amount)?;

    // ========== 7. Update state ==========
    nft_binding.last_pool_claim_week = previous_week;

    match node_type {
        NODE_TYPE_DIAMOND => {
            let new_count = global_state
                .diamond_pool_claimed_count
                .saturating_add(1);
            require!(
                new_count <= DIAMOND_POOL_SHARES as u16,
                NodePoolError::ClaimedCountExceeded
            );
            global_state.diamond_pool_claimed_count = new_count;
        }
        NODE_TYPE_GOLD => {
            let new_count = global_state
                .gold_pool_claimed_count
                .saturating_add(1);
            require!(
                new_count <= GOLD_POOL_SHARES as u16,
                NodePoolError::ClaimedCountExceeded
            );
            global_state.gold_pool_claimed_count = new_count;
        }
        _ => {}
    }

    msg!(
        "NodePoolRewardClaimed: user={}, node_type={}, week={}, amount={}",
        ctx.accounts.user.key(),
        node_type,
        previous_week,
        amount
    );

    emit!(NodePoolRewardClaimed {
        user: ctx.accounts.user.key(),
        node_type,
        week_number: previous_week,
        amount,
    });

    Ok(())
}

/// Query node pool status
pub fn handler_query_node_pool_status(
    ctx: Context<crate::contexts::QueryNodePoolStatus>,
) -> Result<NodePoolStatusResult> {
    let global = &ctx.accounts.global_state;
    let now = Clock::get()?.unix_timestamp;
    let current_week = calculate_week_number(now);

    // Calculate per-share amount
    let diamond_per_share = if DIAMOND_POOL_SHARES > 0 {
        global.diamond_pool_previous / DIAMOND_POOL_SHARES
    } else {
        0
    };
    let gold_per_share = if GOLD_POOL_SHARES > 0 {
        global.gold_pool_previous / GOLD_POOL_SHARES
    } else {
        0
    };

    // Check if user has already claimed
    let user_already_claimed = if let Some(ref binding) = ctx.accounts.nft_binding_state {
        if global.current_week_number > 0 {
            binding.last_pool_claim_week == global.current_week_number - 1
        } else {
            false
        }
    } else {
        false
    };

    Ok(NodePoolStatusResult {
        current_week_number: current_week,
        diamond_pool_current: global.diamond_pool_current,
        gold_pool_current: global.gold_pool_current,
        diamond_pool_previous: global.diamond_pool_previous,
        gold_pool_previous: global.gold_pool_previous,
        diamond_pool_claimed_count: global.diamond_pool_claimed_count,
        gold_pool_claimed_count: global.gold_pool_claimed_count,
        diamond_per_share,
        gold_per_share,
        user_already_claimed,
    })
}

/// Query node pool reward amount for a specific user (read-only)
pub fn handler_query_node_pool_reward(
    ctx: Context<crate::contexts::QueryNodePoolReward>,
) -> Result<NodePoolRewardResult> {
    let global_state = &ctx.accounts.global_state;
    let nft_binding = &ctx.accounts.nft_binding_state;

    // Calculate per-share amounts
    let diamond_per_share = if DIAMOND_POOL_SHARES > 0 {
        global_state.diamond_pool_previous / DIAMOND_POOL_SHARES
    } else {
        0
    };
    let gold_per_share = if GOLD_POOL_SHARES > 0 {
        global_state.gold_pool_previous / GOLD_POOL_SHARES
    } else {
        0
    };

    // Check if there is a claimable week
    let (week_number, is_claimed, reward_amount) = if global_state.current_week_number == 0 {
        // No previous week data
        (0, false, 0)
    } else {
        let previous_week = global_state.current_week_number - 1;
        let is_claimed = nft_binding.last_pool_claim_week == previous_week;

        if is_claimed {
            (previous_week, true, 0)
        } else {
            // Calculate reward amount based on node type
            let amount = match nft_binding.node_type {
                NODE_TYPE_DIAMOND => diamond_per_share,
                NODE_TYPE_GOLD => gold_per_share,
                _ => 0,
            };
            (previous_week, false, amount)
        }
    };

    Ok(NodePoolRewardResult {
        week_number,
        node_type: nft_binding.node_type,
        reward_amount,
        is_claimed,
        diamond_per_share,
        gold_per_share,
    })
}
