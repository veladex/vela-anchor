use anchor_lang::prelude::*;
use crate::{
    structs::*,
    errors::ReferralError,
    zero_copy_storage,
};

/// Generic ancestor node traversal and update function
///
/// # Parameters
/// - `storage_accounts`: Array of references to all 9 storage PDA accounts
/// - `start_parent_id`: Starting parent node ID
/// - `max_depth`: Maximum traversal depth
/// - `program_id`: Program ID, used for PDA verification
/// - `update_fn`: Update closure, takes &mut ReferralData, returns Result<()>
///
/// # Returns
/// - Ok(u32): Number of levels actually updated
/// - Err: If an error is encountered (e.g., circular reference, data corruption, etc.)
///
/// # Example
/// ```ignore
/// // Update total_referrals
/// let updated = traverse_and_update_ancestors(
///     &storage_accounts,
///     parent_id,
///     60,
///     ctx.program_id,
///     |referral| {
///         referral.total_referrals = referral.total_referrals.saturating_add(1);
///         Ok(())
///     },
/// )?;
/// ```
pub fn traverse_and_update_ancestors<F>(
    storage_accounts: &[&UncheckedAccount],
    start_parent_id: u32,
    max_depth: u32,
    program_id: &Pubkey,
    mut update_fn: F,
) -> Result<u32>
where
    F: FnMut(&mut ReferralData) -> Result<()>,
{
    // If starting parent is 0 (root node), return immediately
    if start_parent_id == 0 {
        return Ok(0);
    }

    let mut current_parent_id = start_parent_id;
    let mut updated_count = 0u32;

    // Set for detecting circular references
    let mut visited_ids = Vec::with_capacity(max_depth as usize);

    for depth in 0..max_depth {
        // Check if root node is reached
        if current_parent_id == 0 {
            msg!("Reached root node at depth {}", depth);
            break;
        }

        // Detect circular reference
        if visited_ids.contains(&current_parent_id) {
            msg!("Circular reference detected at ID: {}", current_parent_id);
            return Err(ReferralError::CircularReference.into());
        }
        visited_ids.push(current_parent_id);

        // Decode ID to get PDA index and slot index
        let (pda_index, slot_index) = ReferralStorage::decode_id(current_parent_id);

        // Validate PDA index range
        if pda_index < 1 || pda_index > 9 {
            msg!("Invalid PDA index {} at depth {}, stopping traversal", pda_index, depth);
            break;
        }

        // Get the corresponding storage account
        let storage_account = storage_accounts[(pda_index - 1) as usize];

        // Verify PDA address
        let (expected_pda, _bump) = Pubkey::find_program_address(
            &[ReferralStorage::SEED_PREFIX, &[pda_index]],
            program_id,
        );

        if storage_account.key() != expected_pda {
            msg!("PDA verification failed at depth {}, stopping traversal", depth);
            break;
        }

        // Borrow account data for read/write
        {
            let mut storage_data = storage_account.try_borrow_mut_data()?;

            // Read the current record count
            let count = zero_copy_storage::read_count(&storage_data)?;

            // Validate slot index
            if slot_index >= count {
                msg!("Slot index {} out of bounds (count: {}) at depth {}, stopping traversal",
                     slot_index, count, depth);
                break;
            }

            // Read the current record using zero-copy
            let mut referral = zero_copy_storage::read_record(&storage_data, slot_index)?;

            // Save the next parent ID (before calling update_fn)
            let next_parent_id = referral.parent_id;

            // Call the user-provided update function
            update_fn(&mut referral)?;

            // Write back the updated record using zero-copy
            zero_copy_storage::write_record(&mut storage_data, slot_index, &referral)?;

            msg!("Updated ancestor ID {} at depth {}", current_parent_id, depth);

            // Continue traversing upward
            current_parent_id = next_parent_id;
            updated_count += 1;
        } // storage_data is automatically dropped here
    }

    msg!("Total ancestors updated: {}", updated_count);
    Ok(updated_count)
}

/// Convenience function: Update the total_referrals field of ancestor nodes
///
/// # Parameters
/// - `storage_accounts`: Array of references to all 9 storage PDA accounts
/// - `start_parent_id`: Starting parent node ID
/// - `max_depth`: Maximum traversal depth (recommended: 60)
/// - `program_id`: Program ID
/// - `increment`: Amount to increase (usually 1)
///
/// # Returns
/// - Ok(u32): Number of levels actually updated
pub fn update_ancestors_total_referrals(
    storage_accounts: &[&UncheckedAccount],
    start_parent_id: u32,
    max_depth: u32,
    program_id: &Pubkey,
    increment: u32,
) -> Result<u32> {
    traverse_and_update_ancestors(
        storage_accounts,
        start_parent_id,
        max_depth,
        program_id,
        |referral| {
            referral.total_referrals = referral.total_referrals.saturating_add(increment);
            Ok(())
        },
    )
}

/// Convenience function: Update the total_staked field of ancestor nodes (used for staking)
///
/// # Parameters
/// - `storage_accounts`: Array of references to all 9 storage PDA accounts
/// - `start_parent_id`: Starting parent node ID
/// - `max_depth`: Maximum traversal depth (recommended: 60)
/// - `program_id`: Program ID
/// - `delta`: Staked amount change (positive for increase, negative for decrease)
///
/// # Returns
/// - Ok(u32): Number of levels actually updated
pub fn update_ancestors_total_staked(
    storage_accounts: &[&UncheckedAccount],
    start_parent_id: u32,
    max_depth: u32,
    program_id: &Pubkey,
    delta: i64,
) -> Result<u32> {
    traverse_and_update_ancestors(
        storage_accounts,
        start_parent_id,
        max_depth,
        program_id,
        |referral| {
            if delta >= 0 {
                // Increase stake
                referral.total_staked = referral.total_staked.saturating_add(delta as u64);
            } else {
                // Decrease stake
                referral.total_staked = referral.total_staked.saturating_sub((-delta) as u64);
            }
            Ok(())
        },
    )
}
