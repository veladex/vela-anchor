use anchor_lang::prelude::*;
use crate::{
    constants::*,
    structs::*,
    errors::ReferralError,
    zero_copy_storage,
    events::NodePoolRefreshed,
};

/// Determine level (L0~L7) based on self_staked and total_staked
///
/// Returns the highest level whose requirements are met, checking from L7 downward
pub fn determine_level(self_staked: u64, total_staked: u64) -> u8 {
    for level in (1..COMMUNITY_LEVEL_COUNT).rev() {
        if self_staked >= LEVEL_SELF_STAKED_REQ[level]
            && total_staked >= LEVEL_TOTAL_STAKED_REQ[level]
        {
            return level as u8;
        }
    }
    0 // L0
}

/// Core function for community reward distribution
///
/// Called when a user claims staking interest. Distributes community rewards upward
/// based on base_interest (excluding NFT bonus). All rewards are only recorded in
/// the community_profit field; no immediate transfers are made.
///
/// # Arguments
/// - `storage_accounts`: References to 9 storage PDA accounts
/// - `trigger_user_referral_id`: The referral_id of the triggering user
/// - `reward`: Base interest amount (excluding NFT bonus)
/// - `program_id`: Program ID
///
/// # Returns
/// - Ok(u64): Total amount actually distributed
pub fn distribute_community_reward(
    storage_accounts: &[&AccountInfo],
    trigger_user_referral_id: u32,
    reward: u64,
    program_id: &Pubkey,
    global_state: &mut GlobalState,
    current_timestamp: i64,
) -> Result<u64> {
    if reward == 0 {
        return Ok(0);
    }

    // Calculate direct referral reward and level reward pool
    let direct_reward = (reward as u128)
        .checked_mul(COMMUNITY_DIRECT_REWARD_BPS as u128)
        .unwrap()
        .checked_div(BASIS_POINTS as u128)
        .unwrap() as u64;

    let level_pool = (reward as u128)
        .checked_mul(COMMUNITY_LEVEL_POOL_BPS as u128)
        .unwrap()
        .checked_div(BASIS_POINTS as u128)
        .unwrap() as u64;

    let level_bonus_amount = (reward as u128)
        .checked_mul(COMMUNITY_LEVEL_BONUS_BPS as u128)
        .unwrap()
        .checked_div(BASIS_POINTS as u128)
        .unwrap() as u64;

    let mut total_distributed: u64 = 0;
    let mut level_distributed: u64 = 0;

    // ========== 1. Read the trigger user's parent_id ==========
    let (user_pda_index, user_slot_index) = ReferralStorage::decode_and_validate_id(trigger_user_referral_id)?;
    let user_parent_id = {
        let storage_account = storage_accounts[(user_pda_index - 1) as usize];
        let storage_data = storage_account.try_borrow_data()?;
        let user_data = zero_copy_storage::read_record(&storage_data, user_slot_index)?;
        user_data.parent_id
    };

    if user_parent_id == 0 {
        // Trigger user is root node; no one to receive direct reward; all level rewards go to root itself
        let root_total = level_pool;
        add_team_reward_profit(storage_accounts, ROOT_REFERRAL_ID, root_total)?;
        total_distributed = total_distributed.saturating_add(root_total);
        msg!("Trigger user is root, level_pool {} -> root", root_total);
    } else {
        // ========== 2. Direct referral reward: given to the immediate parent (requires staking) ==========
        if direct_reward > 0 {
            let (parent_pda_index, parent_slot_index) = ReferralStorage::decode_and_validate_id(user_parent_id)?;
            let parent_storage = storage_accounts[(parent_pda_index - 1) as usize];
            let parent_self_staked = {
                let storage_data = parent_storage.try_borrow_data()?;
                let parent_data = zero_copy_storage::read_record(&storage_data, parent_slot_index)?;
                parent_data.self_staked
            };

            if parent_self_staked > 0 {
                add_direct_reward_profit(storage_accounts, user_parent_id, direct_reward)?;
                total_distributed = total_distributed.saturating_add(direct_reward);
                msg!("Direct reward {} -> parent_id {} (staked={})", direct_reward, user_parent_id, parent_self_staked);
            } else {
                // Parent has no active stake; redirect direct reward to root
                add_team_reward_profit(storage_accounts, ROOT_REFERRAL_ID, direct_reward)?;
                total_distributed = total_distributed.saturating_add(direct_reward);
                msg!("Direct reward {} -> root (parent {} has no stake)", direct_reward, user_parent_id);
            }
        }

        // ========== 3. Level reward: traverse upward starting from the immediate parent ==========
        let mut highest_level_seen: u8 = 0;
        // Bitmask: bit N indicates that the same-level bonus for LN has been given (only valid for L3~L7)
        let mut level_bonus_given: u8 = 0;
        let mut current_id = user_parent_id;
        let mut root_found = false;

        // Used to detect circular references
        let mut visited_ids = Vec::with_capacity(COMMUNITY_MAX_TRAVERSE_DEPTH as usize);

        for depth in 0..COMMUNITY_MAX_TRAVERSE_DEPTH {
            if current_id == 0 {
                break;
            }

            // Circular reference detection
            if visited_ids.contains(&current_id) {
                msg!("Circular reference detected at ID: {}", current_id);
                return Err(ReferralError::CircularReference.into());
            }
            visited_ids.push(current_id);

            let (pda_index, slot_index) = ReferralStorage::decode_and_validate_id(current_id)?;

            let storage_account = storage_accounts[(pda_index - 1) as usize];

            // Verify PDA address
            let (expected_pda, _bump) = Pubkey::find_program_address(
                &[ReferralStorage::SEED_PREFIX, &[pda_index]],
                program_id,
            );
            if storage_account.key() != expected_pda {
                msg!("PDA verification failed at depth {}", depth);
                return Err(ReferralError::InvalidStoragePDA.into());
            }

            let (next_parent_id, node_level) = {
                let mut storage_data = storage_account.try_borrow_mut_data()?;
                let mut referral = zero_copy_storage::read_record(&storage_data, slot_index)?;
                let next_parent = referral.parent_id;
                let level = determine_level(referral.self_staked, referral.total_staked);

                let mut node_reward: u64 = 0;

                // Level differential share
                if level > highest_level_seen {
                    let diff_bps = LEVEL_DIFF_BPS[level as usize] - LEVEL_DIFF_BPS[highest_level_seen as usize];
                    let diff_amount = (reward as u128)
                        .checked_mul(diff_bps as u128)
                        .unwrap()
                        .checked_div(BASIS_POINTS as u128)
                        .unwrap() as u64;

                    // Cap to not exceed level_pool
                    let capped = diff_amount.min(level_pool.saturating_sub(level_distributed));
                    if capped > 0 {
                        node_reward = node_reward.saturating_add(capped);
                        level_distributed = level_distributed.saturating_add(capped);
                        msg!("Level diff: id={}, L{} -> L{}, amount={}", current_id, highest_level_seen, level, capped);
                    }

                    highest_level_seen = level;
                } else if level == highest_level_seen && level >= COMMUNITY_LEVEL_BONUS_MIN {
                    // Same-level bonus: same level && L3+ && not yet given for this level
                    let bit = 1u8 << level;
                    if level_bonus_given & bit == 0 {
                        let capped = level_bonus_amount.min(level_pool.saturating_sub(level_distributed));
                        if capped > 0 {
                            node_reward = node_reward.saturating_add(capped);
                            level_distributed = level_distributed.saturating_add(capped);
                            level_bonus_given |= bit;
                            msg!("Level bonus: id={}, L{}, amount={}", current_id, level, capped);
                        }
                    }
                }

                // Write to team_reward_profit
                if node_reward > 0 {
                    referral.team_reward_profit = referral.team_reward_profit.saturating_add(node_reward);
                    zero_copy_storage::write_record(&mut storage_data, slot_index, &referral)?;
                    total_distributed = total_distributed.saturating_add(node_reward);
                }

                // Check if this is the root node
                if next_parent == 0 {
                    root_found = true;
                }

                (next_parent, level)
            };

            // If this is the root node, stop after processing
            if root_found {
                msg!("Reached root node (id={}) at depth {}", current_id, depth);
                break;
            }

            // Check if all level differentials are exhausted and all same-level bonuses have been given
            // Can exit early when highest_level_seen reaches L7(70%) and no more same-level bonuses to give
            // But per the spec, even at L7 we continue traversing to check same-level bonuses, so we continue
            let _ = node_level;

            current_id = next_parent_id;
        }

        // ========== 4. Remainder goes to root ==========
        let remaining = level_pool.saturating_sub(level_distributed);
        if remaining > 0 {
            add_team_reward_profit(storage_accounts, ROOT_REFERRAL_ID, remaining)?;
            total_distributed = total_distributed.saturating_add(remaining);
            msg!("Remaining {} -> root (id={})", remaining, ROOT_REFERRAL_ID);
        }

        msg!("Community reward total distributed: {}, direct={}, level_distributed={}, remaining={}",
            total_distributed, direct_reward, level_distributed, remaining);
    }

    // ========== 5. Node pool accumulation ==========
    let diamond_pool_amount = (reward as u128)
        .checked_mul(NODE_POOL_DIAMOND_BPS as u128).unwrap()
        .checked_div(BASIS_POINTS as u128).unwrap() as u64;
    let gold_pool_amount = (reward as u128)
        .checked_mul(NODE_POOL_GOLD_BPS as u128).unwrap()
        .checked_div(BASIS_POINTS as u128).unwrap() as u64;

    // Cross-week detection & refresh
    maybe_refresh_week(global_state, current_timestamp, storage_accounts)?;

    // Accumulate into the current week
    global_state.diamond_pool_current = global_state.diamond_pool_current.saturating_add(diamond_pool_amount);
    global_state.gold_pool_current = global_state.gold_pool_current.saturating_add(gold_pool_amount);

    total_distributed = total_distributed.saturating_add(diamond_pool_amount).saturating_add(gold_pool_amount);

    msg!("Node pool accumulated: diamond={}, gold={}", diamond_pool_amount, gold_pool_amount);

    Ok(total_distributed)
}

/// Helper function: add direct reward profit
pub(crate) fn add_direct_reward_profit(
    storage_accounts: &[&AccountInfo],
    referral_id: u32,
    amount: u64,
) -> Result<()> {
    let (pda_index, slot_index) = ReferralStorage::decode_and_validate_id(referral_id)?;
    let storage_account = storage_accounts[(pda_index - 1) as usize];
    let mut storage_data = storage_account.try_borrow_mut_data()?;
    let count = zero_copy_storage::read_count(&storage_data)?;
    if slot_index >= count {
        msg!("add_direct_reward_profit: slot_index {} out of bounds (count: {}) for id {}", slot_index, count, referral_id);
        return Err(ReferralError::ReferralNotFound.into());
    }
    let mut referral = zero_copy_storage::read_record(&storage_data, slot_index)?;
    referral.direct_reward_profit = referral.direct_reward_profit.saturating_add(amount);
    zero_copy_storage::write_record(&mut storage_data, slot_index, &referral)?;
    Ok(())
}

/// Helper function: add team reward profit
pub(crate) fn add_team_reward_profit(
    storage_accounts: &[&AccountInfo],
    referral_id: u32,
    amount: u64,
) -> Result<()> {
    let (pda_index, slot_index) = ReferralStorage::decode_and_validate_id(referral_id)?;
    let storage_account = storage_accounts[(pda_index - 1) as usize];
    let mut storage_data = storage_account.try_borrow_mut_data()?;
    let count = zero_copy_storage::read_count(&storage_data)?;
    if slot_index >= count {
        msg!("add_team_reward_profit: slot_index {} out of bounds (count: {}) for id {}", slot_index, count, referral_id);
        return Err(ReferralError::ReferralNotFound.into());
    }
    let mut referral = zero_copy_storage::read_record(&storage_data, slot_index)?;
    referral.team_reward_profit = referral.team_reward_profit.saturating_add(amount);
    zero_copy_storage::write_record(&mut storage_data, slot_index, &referral)?;
    Ok(())
}

/// Cross-week detection & refresh (逐周滚动版本)
///
/// If the current time has crossed into a new week, automatically loop through
/// each skipped week to ensure correct forfeiture and pool rotation:
/// 1. Add unclaimed rewards from the previous week to the root referrer's community_profit
/// 2. Move current week data to the previous week
/// 3. Reset current week data to zero
/// 4. Advance current_week_number by one
/// Repeats until caught up to the real current week.
pub(crate) fn maybe_refresh_week(
    global: &mut GlobalState,
    now: i64,
    storage_accounts: &[&AccountInfo],
) -> Result<()> {
    let current_week = calculate_week_number(now);

    if current_week <= global.current_week_number {
        return Ok(()); // Same week, no refresh needed
    }

    // 限制单次交易最多处理的跳周数，防止 CU 溢出
    let weeks_to_process = (current_week - global.current_week_number).min(MAX_WEEKS_PER_REFRESH);

    // 逐周处理每一个跨越的周
    for _i in 0..weeks_to_process {
        let processing_week = global.current_week_number + 1;

        // 1. Calculate unclaimed amount from the previous week
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

        let diamond_claimed_total = diamond_per_share * global.diamond_pool_claimed_count as u64;
        let gold_claimed_total = gold_per_share * global.gold_pool_claimed_count as u64;

        let diamond_unclaimed = global.diamond_pool_previous.saturating_sub(diamond_claimed_total);
        let gold_unclaimed = global.gold_pool_previous.saturating_sub(gold_claimed_total);

        // 2. Add unclaimed amounts to the root referrer's team_reward_profit
        let total_unclaimed = diamond_unclaimed.saturating_add(gold_unclaimed);
        if total_unclaimed > 0 {
            add_team_reward_profit(storage_accounts, ROOT_REFERRAL_ID, total_unclaimed)?;
            msg!("Week {} unclaimed {} -> root", global.current_week_number, total_unclaimed);
        }

        // 3. Move current week to previous week
        global.diamond_pool_previous = global.diamond_pool_current;
        global.gold_pool_previous = global.gold_pool_current;

        // 4. Reset current week to zero
        global.diamond_pool_current = 0;
        global.gold_pool_current = 0;
        global.diamond_pool_claimed_count = 0;
        global.gold_pool_claimed_count = 0;

        // 5. Advance one week
        global.current_week_number = processing_week;

        emit!(NodePoolRefreshed {
            week_number: processing_week,
            diamond_unclaimed_to_root: diamond_unclaimed,
            gold_unclaimed_to_root: gold_unclaimed,
            diamond_new_pool: global.diamond_pool_previous,
            gold_new_pool: global.gold_pool_previous,
        });

        // 优化：如果 current 和 previous 都为 0，剩余周无需逐一处理
        if global.diamond_pool_current == 0
            && global.gold_pool_current == 0
            && global.diamond_pool_previous == 0
            && global.gold_pool_previous == 0
        {
            global.current_week_number = current_week;
            break;
        }
    }

    Ok(())
}

/// Calculate the current UTC week number
pub(crate) fn calculate_week_number(timestamp: i64) -> u64 {
    ((timestamp - WEEK_EPOCH_OFFSET) / SECONDS_PER_WEEK) as u64
}
