use anchor_lang::prelude::*;
use crate::{
    contexts::*,
    structs::*,
    errors::ReferralError,
    events::ReferralBindingEvent,
    referral_utils,
    constants::{REFERRAL_UPDATE_LEVELS, REFERRAL_REGISTRATION_FEE},
};

/// Get total referral count in the system
fn get_total_referral_count(accounts: &AddReferral) -> Result<u32> {
    let storage_accounts = [
        &accounts.storage_1,
        &accounts.storage_2,
        &accounts.storage_3,
        &accounts.storage_4,
        &accounts.storage_5,
        &accounts.storage_6,
        &accounts.storage_7,
        &accounts.storage_8,
        &accounts.storage_9,
    ];

    let mut total = 0u32;

    for storage_account in storage_accounts.iter() {
        // Read only the count field
        let data = storage_account.try_borrow_data()?;
        if data.len() >= 13 {
            // Read count: offset = 8(discriminator) + 1(index) = 9
            let count = u32::from_le_bytes([data[9], data[10], data[11], data[12]]);
            total = total.saturating_add(count);
        }
    }

    Ok(total)
}

/// Verify if a referrer ID exists
fn verify_referral_exists(
    referral_id: u32,
    accounts: &AddReferral,
    program_id: &Pubkey,
) -> Result<bool> {
    // Safe decode with canonical validation
    let (pda_index, slot_index) = match ReferralStorage::decode_and_validate_id(referral_id) {
        Ok(result) => result,
        Err(_) => return Ok(false),  // non-canonical ID treated as non-existent
    };

    // Get the corresponding storage account
    let storage_accounts = [
        &accounts.storage_1,
        &accounts.storage_2,
        &accounts.storage_3,
        &accounts.storage_4,
        &accounts.storage_5,
        &accounts.storage_6,
        &accounts.storage_7,
        &accounts.storage_8,
        &accounts.storage_9,
    ];

    let storage_account = storage_accounts[(pda_index - 1) as usize];

    // Verify PDA
    let (expected_pda, _bump) = Pubkey::find_program_address(
        &[ReferralStorage::SEED_PREFIX, &[pda_index]],
        program_id,
    );

    if storage_account.key() != expected_pda {
        return Ok(false);
    }

    // Zero-copy read count
    use crate::zero_copy_storage;

    let storage_data = storage_account.try_borrow_data()?;

    // Zero-copy read count
    let count = zero_copy_storage::read_count(&storage_data).unwrap_or(0);

    // Check if slot_index is in valid range
    Ok(slot_index < count)
}



/// Initialize ReferralManager instruction handler with root user
pub fn handler_initialize(
    ctx: Context<InitializeReferralManager>,
    root_wallet: Pubkey,
) -> Result<()> {
    use crate::zero_copy_storage;
    use anchor_lang::solana_program::program::invoke;
    use anchor_lang::solana_program::system_instruction;

    let clock = Clock::get()?;

    // Initialize manager
    let manager = &mut ctx.accounts.manager;
    manager.authority = ctx.accounts.authority.key();
    manager.current_pda_index = 1;
    manager.initialized = true;

    // Initialize 9 storage PDAs (zero-copy design)
    // Only set the index field, count will be set via zero-copy when writing data
    ctx.accounts.storage_1.index = 1;
    ctx.accounts.storage_1.reserved = [0; 3];

    ctx.accounts.storage_2.index = 2;
    ctx.accounts.storage_2.reserved = [0; 3];

    ctx.accounts.storage_3.index = 3;
    ctx.accounts.storage_3.reserved = [0; 3];

    ctx.accounts.storage_4.index = 4;
    ctx.accounts.storage_4.reserved = [0; 3];

    ctx.accounts.storage_5.index = 5;
    ctx.accounts.storage_5.reserved = [0; 3];

    ctx.accounts.storage_6.index = 6;
    ctx.accounts.storage_6.reserved = [0; 3];

    ctx.accounts.storage_7.index = 7;
    ctx.accounts.storage_7.reserved = [0; 3];

    ctx.accounts.storage_8.index = 8;
    ctx.accounts.storage_8.reserved = [0; 3];

    ctx.accounts.storage_9.index = 9;
    ctx.accounts.storage_9.reserved = [0; 3];

    msg!("ReferralManager initialized with 9 storage PDAs (zero-copy)");

    // ============== Add root user ==============
    // Create root user data
    let root_data = ReferralData {
        wallet: root_wallet,
        parent_id: 0,  // Root node
        created_at: clock.unix_timestamp,
        total_referrals: 0,
        total_staked: 0,
        self_staked: 0,
        direct_reward_profit: 0,
        team_reward_profit: 0,
    };

    // Get storage_1 account info and manually handle zero-copy write
    let storage_1_info = ctx.accounts.storage_1.to_account_info();

    // Check if we need to expand storage for one record
    let required_size = zero_copy_storage::HEADER_SIZE + zero_copy_storage::RECORD_SIZE;
    let current_size = storage_1_info.data_len();

    if current_size < required_size {
        let rent = Rent::get()?;
        let new_minimum_balance = rent.minimum_balance(required_size);
        let lamports_diff = new_minimum_balance.saturating_sub(storage_1_info.lamports());

        if lamports_diff > 0 {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.authority.key,
                    storage_1_info.key,
                    lamports_diff,
                ),
                &[
                    ctx.accounts.authority.to_account_info(),
                    storage_1_info.clone(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }

        storage_1_info.resize(required_size)?;
    }

    // Write root user record using zero-copy
    {
        let mut storage_data = storage_1_info.try_borrow_mut_data()?;

        // Initialize the header (index, count=0, reserved)
        zero_copy_storage::init_header(&mut storage_data, 1)?;

        // Write root user record at index 0
        zero_copy_storage::write_record(&mut storage_data, 0, &root_data)?;

        // Update count to 1
        zero_copy_storage::update_count(&mut storage_data, 1)?;
    }

    // Update the Account object to reflect the changes
    ctx.accounts.storage_1.count = 1;

    // Generate root user ID: storage_index * 1000000 + slot_index = 1 * 1000000 + 0
    let root_id = 1_000_000u32;

    // Create wallet ID mapping for root user
    let wallet_mapping = &mut ctx.accounts.wallet_mapping;
    wallet_mapping.wallet = root_wallet;
    wallet_mapping.referral_id = root_id;

    msg!("Root user created with ID: {} (wallet: {})", root_id, root_wallet);

    // ============== Emit ReferralBindingEvent ==============
    emit!(ReferralBindingEvent {
        parent_wallet: Pubkey::default(),  // Root node has no parent
        parent_id: 0,
        my_wallet: root_wallet,
        my_id: root_id,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Add referrer instruction handler (automatically select available storage)
pub fn handler_add_referral(
    ctx: Context<AddReferral>,
    wallet: Pubkey,
    parent_id: u32,
) -> Result<u32> {
    use anchor_lang::solana_program::program::invoke;
    use anchor_lang::solana_program::system_instruction;

    let clock = Clock::get()?;

    // ============== 收取推荐人注册费（0.01 SOL） ==============
    let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.wallet_signer.key(),
        &ctx.accounts.referral_fee_wallet.key(),
        REFERRAL_REGISTRATION_FEE,
    );
    anchor_lang::solana_program::program::invoke(
        &transfer_ix,
        &[
            ctx.accounts.wallet_signer.to_account_info(),
            ctx.accounts.referral_fee_wallet.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;
    msg!("Referral registration fee paid: {} lamports", REFERRAL_REGISTRATION_FEE);

    // ============== New: Root node uniqueness check ==============
    if parent_id == 0 {
        // parent_id=0 means to become a root node
        // Check if any user exists in the entire system
        let total_count = get_total_referral_count(&ctx.accounts)?;

        if total_count > 0 {
            // Already have users, cannot create another root node
            return Err(ReferralError::RootNodeAlreadyExists.into());
        }

        msg!("Creating system root node, wallet: {}", wallet);
    } else {
        // ============== New: Verify if parent_id exists ==============
        // Need to verify that the parent_id actually exists
        let parent_exists = verify_referral_exists(
            parent_id,
            &ctx.accounts,
            ctx.program_id,
        )?;

        if !parent_exists {
            return Err(ReferralError::ParentNotFound.into());
        }

        msg!("Verification passed: parent_id {} exists", parent_id);
    }

    // Create referrer data
    let referral_data = ReferralData {
        wallet,
        parent_id,
        created_at: clock.unix_timestamp,
        total_referrals: 0,
        total_staked: 0,
        self_staked: 0,
        direct_reward_profit: 0,
        team_reward_profit: 0,
    };

    // Get manager (can mutate after verification)
    let manager = &mut ctx.accounts.manager;

    // Get all storage accounts
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

    // Pre-validate all 9 storage PDAs before searching for space
    // This ensures all passed accounts are legitimate even if storage_1 still has space
    for idx in 1u8..=9u8 {
        let account = storage_accounts[(idx - 1) as usize];
        let (expected_pda, _bump) = Pubkey::find_program_address(
            &[ReferralStorage::SEED_PREFIX, &[idx]],
            ctx.program_id,
        );
        require_keys_eq!(
            account.key(),
            expected_pda,
            ReferralError::InvalidStoragePDA
        );
    }

    // Find the first storage with space (PDAs already validated above)
    let mut used_idx: u8 = 0;
    let mut storage_account = None;

    for idx in 1u8..=9u8 {
        let account = storage_accounts[(idx - 1) as usize];

        // Check if there is space (only read count field)
        let has_space = {
            let data = account.try_borrow_data()?;
            // Read count field: 8 (discriminator) + 1 (index) = 9 bytes offset
            if data.len() < 13 {
                return Err(ReferralError::InvalidPdaIndex.into());
            }
            let count = u32::from_le_bytes([data[9], data[10], data[11], data[12]]);
            count < ReferralStorage::MAX_CAPACITY
        };

        if has_space {
            used_idx = idx;
            storage_account = Some(account);
            msg!("Found available Storage PDA {}", idx);
            break;
        }
    }

    // If all storages are full
    if storage_account.is_none() {
        return Err(ReferralError::AllStoragesFull.into());
    }

    let storage_account = storage_account.unwrap();

    // Update manager's current index to the found one
    manager.current_pda_index = used_idx;

    // Use zero-copy architecture to add record
    let id = {
        use crate::zero_copy_storage;

        let mut storage_data = storage_account.try_borrow_mut_data()?;

        // Zero-copy read count
        let current_count = zero_copy_storage::read_count(&storage_data)?;
        let id = used_idx as u32 * 1000000 + current_count;

        // Calculate new size
        let old_size = storage_data.len();
        let new_size = zero_copy_storage::HEADER_SIZE + ((current_count + 1) as usize) * zero_copy_storage::RECORD_SIZE;

        // If need to expand space
        if new_size > old_size {
            drop(storage_data);

            let rent = Rent::get()?;
            let new_minimum_balance = rent.minimum_balance(new_size);
            let lamports_diff = new_minimum_balance.saturating_sub(storage_account.lamports());

            if lamports_diff > 0 {
                invoke(
                    &system_instruction::transfer(
                        ctx.accounts.payer.key,
                        storage_account.key,
                        lamports_diff,
                    ),
                    &[
                        ctx.accounts.payer.to_account_info(),
                        storage_account.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                    ],
                )?;
            }

            storage_account.resize(new_size)?;
            storage_data = storage_account.try_borrow_mut_data()?;
        }

        // Zero-copy write new record (only operate 56 bytes)
        zero_copy_storage::write_record(&mut storage_data, current_count, &referral_data)?;

        // Zero-copy update count
        zero_copy_storage::update_count(&mut storage_data, current_count + 1)?;

        id
    };

    // ============== New: Initialize wallet ID mapping ==============
    let wallet_mapping = &mut ctx.accounts.wallet_mapping;
    wallet_mapping.wallet = wallet;
    wallet_mapping.referral_id = id;

    msg!("Created wallet mapping: {} -> {}", wallet, id);
    msg!("Added referral with ID: {} (storage {})", id, used_idx);

    // ============== Emit binding event ==============
    // Get parent wallet address
    let parent_wallet = if parent_id == 0 {
        // Root node, parent is set to system address (all zeros)
        Pubkey::default()
    } else {
        // Query parent's wallet address using zero-copy storage
        let (parent_pda_index, parent_slot_index) = ReferralStorage::decode_and_validate_id(parent_id)?;
        let parent_storage_account = storage_accounts[(parent_pda_index - 1) as usize];
        let parent_storage_data = parent_storage_account.try_borrow_data()?;

        // Use zero_copy_storage module to correctly read wallet address
        // This uses the correct HEADER_SIZE (16 bytes) instead of incorrect 17 bytes
        use crate::zero_copy_storage;
        zero_copy_storage::read_wallet_at(&parent_storage_data, parent_slot_index)
            .unwrap_or(Pubkey::default())
    };

    // Emit binding event
    emit!(ReferralBindingEvent {
        parent_wallet,
        parent_id,
        my_wallet: wallet,
        my_id: id,
        timestamp: clock.unix_timestamp,
    });

    // ============== Update ancestors' total_referrals ==============
    // Update all ancestor nodes to increment their total_referrals
    if parent_id != 0 {
        msg!("Starting ancestor update for parent_id: {}", parent_id);

        let updated_count = referral_utils::update_ancestors_total_referrals(
            &storage_accounts,
            parent_id,
            REFERRAL_UPDATE_LEVELS, // Use constant instead of hardcoded value
            ctx.program_id,
            1,  // Increment by 1
        )?;

        msg!("Successfully updated {} ancestor nodes", updated_count);
    } else {
        msg!("Root node, no ancestors to update");
    }

    Ok(id)
}

/// Get referrer information handler function
pub fn handler_get_referral(
    ctx: Context<GetReferral>,
    referral_id: u32,
) -> Result<ReferralData> {
    // Decode ID to get PDA index and slot index
    let (pda_index, slot_index) = ReferralStorage::decode_and_validate_id(referral_id)?;

    // Get all storage accounts
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

    // Get the corresponding storage account
    let storage_account = storage_accounts[(pda_index - 1) as usize];

    // Verify PDA
    let (expected_pda, _bump) = Pubkey::find_program_address(
        &[ReferralStorage::SEED_PREFIX, &[pda_index]],
        ctx.program_id,
    );
    require_keys_eq!(
        storage_account.key(),
        expected_pda,
        ReferralError::InvalidPdaIndex
    );

    // Use zero-copy to read record
    use crate::zero_copy_storage;

    let storage_data = storage_account.try_borrow_data()?;

    // Zero-copy read count
    let count = zero_copy_storage::read_count(&storage_data)?;

    // Check if slot_index is valid
    if slot_index >= count {
        return Err(ReferralError::ReferralNotFound.into());
    }

    // Zero-copy read single record (only deserialize 56 bytes)
    let referral = zero_copy_storage::read_record(&storage_data, slot_index)?;

    msg!("Found referral with ID: {} (storage {}, slot {})", referral_id, pda_index, slot_index);

    Ok(referral)
}

/// Get referrer ID by wallet address
pub fn handler_get_wallet_id(
    ctx: Context<GetWalletId>,
    wallet: Pubkey,
) -> Result<Option<u32>> {
    // Verify PDA
    let (expected_pda, _bump) = Pubkey::find_program_address(
        &[WalletIdMapping::SEED_PREFIX, wallet.as_ref()],
        ctx.program_id,
    );

    // If PDA address doesn't match, account doesn't exist
    if ctx.accounts.wallet_mapping.key() != expected_pda {
        msg!("Wallet mapping not found for: {}", wallet);
        return Ok(None);
    }

    // Check if account exists and is initialized
    let account_info = ctx.accounts.wallet_mapping.to_account_info();
    if account_info.data_is_empty() || account_info.owner != ctx.program_id {
        msg!("Wallet mapping not found for: {}", wallet);
        return Ok(None);
    }

    // Read mapping data
    let data = account_info.try_borrow_data()?;
    if data.len() < WalletIdMapping::SIZE {
        msg!("Invalid wallet mapping data for: {}", wallet);
        return Ok(None);
    }

    let mapping: WalletIdMapping = AnchorDeserialize::deserialize(&mut &data[8..])?;

    msg!("Found wallet mapping: {} -> {}", wallet, mapping.referral_id);
    Ok(Some(mapping.referral_id))
}

/// Get complete wallet information (wallet -> referral_id -> ReferralData + parent_wallet)
pub fn handler_get_wallet_info(
    ctx: Context<GetWalletInfo>,
    wallet: Pubkey,
) -> Result<Option<WalletInfoResult>> {
    // Step 1: Get referral_id from wallet address
    let (expected_pda, _bump) = Pubkey::find_program_address(
        &[WalletIdMapping::SEED_PREFIX, wallet.as_ref()],
        ctx.program_id,
    );

    // If PDA address doesn't match, account doesn't exist
    if ctx.accounts.wallet_mapping.key() != expected_pda {
        msg!("Wallet mapping not found for: {}", wallet);
        return Ok(None);
    }

    // Check if account exists and is initialized
    let account_info = ctx.accounts.wallet_mapping.to_account_info();
    if account_info.data_is_empty() || account_info.owner != ctx.program_id {
        msg!("Wallet mapping not found for: {}", wallet);
        return Ok(None);
    }

    // Read mapping data
    let data = account_info.try_borrow_data()?;
    if data.len() < WalletIdMapping::SIZE {
        msg!("Invalid wallet mapping data for: {}", wallet);
        return Ok(None);
    }

    let mapping: WalletIdMapping = AnchorDeserialize::deserialize(&mut &data[8..])?;
    let referral_id = mapping.referral_id;

    msg!("Found wallet mapping: {} -> {}", wallet, referral_id);

    // Step 2: Get ReferralData using referral_id
    // Safe decode with canonical validation
    let (pda_index, slot_index) = match ReferralStorage::decode_and_validate_id(referral_id) {
        Ok(result) => result,
        Err(_) => {
            msg!("Invalid referral_id: {}", referral_id);
            return Ok(None);
        }
    };

    // Get all storage accounts
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

    // Get the corresponding storage account
    let storage_account = storage_accounts[(pda_index - 1) as usize];

    // Verify PDA
    let (expected_storage_pda, _bump) = Pubkey::find_program_address(
        &[ReferralStorage::SEED_PREFIX, &[pda_index]],
        ctx.program_id,
    );

    if storage_account.key() != expected_storage_pda {
        msg!("Invalid storage PDA for index: {}", pda_index);
        return Ok(None);
    }

    // Use zero-copy to read record
    use crate::zero_copy_storage;

    let storage_data = storage_account.try_borrow_data()?;

    // Zero-copy read count
    let count = zero_copy_storage::read_count(&storage_data)?;

    // Check if slot_index is valid
    if slot_index >= count {
        msg!("Invalid slot index: {} (count: {})", slot_index, count);
        return Ok(None);
    }

    // Zero-copy read single record
    let referral = zero_copy_storage::read_record(&storage_data, slot_index)?;

    // Step 3: Get parent wallet address
    let parent_wallet = if referral.parent_id == 0 {
        // Root node, no parent
        Pubkey::default()
    } else {
        // Query parent's wallet address
        // Safe decode with canonical validation
        let (parent_pda_index, parent_slot_index) = match ReferralStorage::decode_and_validate_id(referral.parent_id) {
            Ok(result) => result,
            Err(_) => {
                msg!("Invalid parent PDA index from parent_id: {}", referral.parent_id);
                return Ok(Some(WalletInfoResult {
                    wallet: referral.wallet,
                    referral_id,
                    parent_id: referral.parent_id,
                    parent_wallet: Pubkey::default(),
                    created_at: referral.created_at,
                    total_referrals: referral.total_referrals,
                    total_staked: referral.total_staked,
                }));
            }
        };

        {
            let parent_storage_account = storage_accounts[(parent_pda_index - 1) as usize];
            let parent_storage_data = parent_storage_account.try_borrow_data()?;

            // Zero-copy read parent's count
            let parent_count = zero_copy_storage::read_count(&parent_storage_data)?;

            // Check if parent slot_index is valid
            if parent_slot_index >= parent_count {
                msg!("Invalid parent slot index: {} (count: {})", parent_slot_index, parent_count);
                Pubkey::default()
            } else {
                // Zero-copy read parent record
                let parent_referral = zero_copy_storage::read_record(&parent_storage_data, parent_slot_index)?;
                parent_referral.wallet
            }
        }
    };

    msg!("Found complete wallet info for: {} (ID: {}, storage: {}, slot: {}, parent_wallet: {})",
         wallet, referral_id, pda_index, slot_index, parent_wallet);

    // Step 4: Construct WalletInfoResult
    let result = WalletInfoResult {
        wallet: referral.wallet,
        referral_id,
        parent_id: referral.parent_id,
        parent_wallet,
        created_at: referral.created_at,
        total_referrals: referral.total_referrals,
        total_staked: referral.total_staked,
    };

    Ok(Some(result))
}
