use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::{BindingError, TokenReleaseError};
use crate::structs::*;
use crate::events::*;
use crate::contexts::{BindNft, VerifyBinding, RebindNft, UnbindNft, ClaimReleasedTokens, QueryReleasableTokens};

// ============================================================================
// Helper functions
// ============================================================================

/// Verify NFT collection using already-deserialized MetadataAccount
fn verify_nft_collection(
    metadata: &anchor_spl::metadata::MetadataAccount,
    expected_collection: &Pubkey,
) -> Result<()> {
    // Check collection field exists
    if let Some(collection) = &metadata.collection {
        // Verify collection mint matches expected
        require!(
            collection.key == *expected_collection,
            BindingError::InvalidNodeCollection
        );
        // Verify collection is verified by authority
        require!(
            collection.verified,
            BindingError::InvalidNodeCollection
        );
        Ok(())
    } else {
        // No collection field found
        Err(error!(BindingError::InvalidNodeCollection))
    }
}

/// Verify user owns the NFT
fn verify_nft_ownership(
    user_token_account: &Account<TokenAccount>,
    user: &Pubkey,
    nft_mint: &Pubkey,
) -> Result<()> {
    // Verify token account belongs to user
    require!(
        user_token_account.owner == *user,
        BindingError::NotNftOwner
    );

    // Verify token account is for the correct NFT mint
    require!(
        user_token_account.mint == *nft_mint,
        BindingError::NotNftOwner
    );

    // Verify user has at least 1 NFT
    require!(
        user_token_account.amount >= 1,
        BindingError::NotNftOwner
    );

    Ok(())
}

/// Calculate releasable token amount
fn calculate_releasable_amount(
    nft_binding_state: &NftBindingState,
    current_timestamp: i64,
) -> Result<(u64, u64)> {
    // 1. Calculate binding days
    let time_elapsed = current_timestamp.saturating_sub(nft_binding_state.initial_bound_at);
    let binding_days = (time_elapsed as u64) / SECONDS_PER_DAY;

    // 2. Get daily release amount based on node type
    let daily_release = match nft_binding_state.node_type {
        NODE_TYPE_DIAMOND => DIAMOND_DAILY_RELEASE,
        NODE_TYPE_GOLD => GOLD_DAILY_RELEASE,
        _ => return Err(error!(BindingError::InvalidNodeCollection)),
    };

    // 3. Calculate total theoretical release
    let theoretical_release = binding_days
        .checked_mul(daily_release)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // 4. Apply total release limit
    let actual_total_release = theoretical_release.min(nft_binding_state.total_release);

    // 5. Calculate current releasable amount
    let releasable_amount = actual_total_release.saturating_sub(nft_binding_state.released_amount);

    Ok((releasable_amount, binding_days))
}

// ============================================================================
// Instruction handlers
// ============================================================================

/// Handler for bind_nft instruction
pub fn handler_bind_nft(
    ctx: Context<BindNft>,
) -> Result<()> {
    let user = &ctx.accounts.user;
    let nft_mint = &ctx.accounts.nft_mint;
    let user_token_account = &ctx.accounts.user_token_account;
    let nft_metadata = &ctx.accounts.nft_metadata;
    let nft_binding_state = &mut ctx.accounts.nft_binding_state;
    let user_state = &mut ctx.accounts.user_state;
    let diamond_collection_state = &ctx.accounts.diamond_collection_state;
    let gold_collection_state = &ctx.accounts.gold_collection_state;
    let clock = Clock::get()?;

    // 1. Verify user owns the NFT
    verify_nft_ownership(user_token_account, &user.key(), &nft_mint.key())?;

    // 2. Verify NFT collection and determine node type
    // Try Diamond collection first
    let (node_type, total_release) = if verify_nft_collection(
        nft_metadata,
        &diamond_collection_state.collection_mint
    ).is_ok() {
        (NODE_TYPE_DIAMOND, DIAMOND_TOTAL_RELEASE)
    }
    // Try Gold collection
    else if verify_nft_collection(
        nft_metadata,
        &gold_collection_state.collection_mint
    ).is_ok() {
        (NODE_TYPE_GOLD, GOLD_TOTAL_RELEASE)
    }
    // Neither collection matched
    else {
        return Err(error!(BindingError::InvalidNodeCollection));
    };

    // 4. Check UserState: user shouldn't already have an active binding
    if user_state.bound_nft_mint != Pubkey::default() {
        // Check if old binding is still valid (old NFT still in wallet)
        // If valid, reject the new binding
        // If invalid, allow overwrite

        // For now, we'll be strict and require the user to have no binding
        // In production, you might want to check the old NFT token account
        return Err(error!(BindingError::UserAlreadyBound));
    }

    // 5. Initialize NftBindingState
    nft_binding_state.nft_mint = nft_mint.key();
    nft_binding_state.owner = user.key();
    nft_binding_state.node_type = node_type;
    nft_binding_state.total_release = total_release;
    nft_binding_state.released_amount = 0;
    // 检查是否为预售 NFT，若是则使用预设时间，否则使用当前时间
    let initial_bound_at = crate::nft_saletime::get_presale_bound_time(&nft_mint.key())
        .unwrap_or(clock.unix_timestamp);
    nft_binding_state.initial_bound_at = initial_bound_at;
    nft_binding_state.last_bound_at = initial_bound_at;
    nft_binding_state.bump = ctx.bumps.nft_binding_state;

    // 6. Update UserState
    user_state.bound_nft_mint = nft_mint.key();
    user_state.bump = ctx.bumps.user_state;

    msg!("NFT bound successfully: mint={}, owner={}, node_type={}, total_release={}",
        nft_mint.key(), user.key(), node_type, total_release);

    Ok(())
}

/// Handler for unbind_nft instruction
/// Allows a user to clear their NFT binding when they no longer hold the NFT
pub fn handler_unbind_nft(ctx: Context<UnbindNft>) -> Result<()> {
    let user = &ctx.accounts.user;
    let user_state = &mut ctx.accounts.user_state;
    let nft_binding_state = &mut ctx.accounts.nft_binding_state;
    let user_token_account = &ctx.accounts.user_token_account;

    // 1. Confirm user_state has an active binding
    require!(
        user_state.bound_nft_mint != Pubkey::default(),
        BindingError::BindingNotFound
    );

    // 2. Confirm nft_binding_state corresponds to the bound NFT
    require!(
        nft_binding_state.nft_mint == user_state.bound_nft_mint,
        BindingError::BindingNotFound
    );

    // 3. Confirm nft_binding_state owner is the current user
    require!(
        nft_binding_state.owner == user.key(),
        BindingError::NotNftOwner
    );

    // 4. Verify user no longer holds the NFT (amount must be 0)
    require!(
        user_token_account.amount == 0,
        BindingError::OwnerStillHoldsNft
    );

    // 5. Check binding duration (must be at least 15 days)
    let clock = Clock::get()?;
    let time_since_bind = clock.unix_timestamp - nft_binding_state.last_bound_at;
    require!(
        time_since_bind >= BINDING_COOLDOWN_SECONDS,
        BindingError::UnbindCooldownNotComplete
    );

    // 6. Clear UserState binding
    let nft_mint = user_state.bound_nft_mint;
    user_state.bound_nft_mint = Pubkey::default();

    // Note: Do NOT clear nft_binding_state.owner here.
    // rebind_nft context derives old_owner_state PDA from nft_binding_state.owner,
    // so we must keep it to allow future rebind by the new NFT holder.
    // The cleared user_state.bound_nft_mint is sufficient to indicate unbind.

    msg!("NFT unbound by user: user={}, nft_mint={}", user.key(), nft_mint);

    Ok(())
}

/// Handler for verify_binding instruction
pub fn handler_verify_binding(
    ctx: Context<VerifyBinding>,
) -> Result<BindingVerificationResult> {
    let user = &ctx.accounts.user;
    let nft_mint = &ctx.accounts.nft_mint;
    let user_token_account = &ctx.accounts.user_token_account;
    let nft_binding_account = &ctx.accounts.nft_binding_state;

    // Verify the PDA address is correct
    let (expected_pda, _bump) = Pubkey::find_program_address(
        &[NFT_BINDING_SEED, nft_mint.key().as_ref()],
        ctx.program_id,
    );
    require_keys_eq!(nft_binding_account.key(), expected_pda, BindingError::InvalidPdaAddress);

    // Check if the account has data (exists and is initialized)
    let account_data = nft_binding_account.try_borrow_data()?;
    if account_data.len() == 0 || nft_binding_account.owner == &anchor_lang::solana_program::system_program::ID {
        // PDA doesn't exist, NFT has never been bound
        return Ok(BindingVerificationResult {
            has_binding: false,
            is_active: false,
            binding_info: None,
        });
    }

    // Verify the account is owned by this program
    require_keys_eq!(*nft_binding_account.owner, ctx.program_id.key(), BindingError::InvalidPdaAddress);

    // Deserialize the NftBindingState from account data (skip 8-byte discriminator)
    let nft_binding_state = NftBindingState::try_deserialize(&mut &account_data[..])?;

    // PDA exists, now check if binding is active
    let owner_matches = nft_binding_state.owner == user.key();

    // Verify user still owns the NFT
    let nft_in_wallet = user_token_account.owner == user.key()
        && user_token_account.mint == nft_mint.key()
        && user_token_account.amount >= 1;

    let is_active = owner_matches && nft_in_wallet;

    // Prepare binding info
    let binding_info = Some(BindingInfo {
        nft_mint: nft_binding_state.nft_mint,
        owner: nft_binding_state.owner,
        node_type: nft_binding_state.node_type,
        total_release: nft_binding_state.total_release,
        released_amount: nft_binding_state.released_amount,
        initial_bound_at: nft_binding_state.initial_bound_at,
        last_bound_at: nft_binding_state.last_bound_at,
    });

    Ok(BindingVerificationResult {
        has_binding: true,
        is_active,
        binding_info,
    })
}

/// Handler for rebind_nft instruction
pub fn handler_rebind_nft(
    ctx: Context<RebindNft>,
) -> Result<()> {
    let user = &ctx.accounts.user;
    let nft_mint = &ctx.accounts.nft_mint;
    let user_token_account = &ctx.accounts.user_token_account;
    let nft_binding_state = &mut ctx.accounts.nft_binding_state;
    let user_state = &mut ctx.accounts.user_state;
    let old_owner_state = &mut ctx.accounts.old_owner_state;
    let clock = Clock::get()?;

    // 1. Verify new owner holds the NFT
    verify_nft_ownership(user_token_account, &user.key(), &nft_mint.key())?;

    // 2. Verify binding PDA exists and matches the NFT
    require!(
        nft_binding_state.nft_mint == nft_mint.key(),
        BindingError::BindingNotFound
    );

    // 3. Verify old owner no longer holds the NFT
    // We need to check the old owner's token account
    // For simplicity, we'll just check that the current user is NOT the old owner
    let old_owner = nft_binding_state.owner;
    require!(
        old_owner != user.key(),
        BindingError::OwnerStillHoldsNft
    );

    // Note: In production, you might want to verify the old owner's token account
    // to ensure they actually don't have the NFT anymore

    // 4. Check cooldown period (15 days since last binding)
    let time_since_last_bind = clock.unix_timestamp - nft_binding_state.last_bound_at;
    require!(
        time_since_last_bind >= BINDING_COOLDOWN_SECONDS,
        BindingError::BindingCooldown
    );

    // 5. Check new owner's UserState
    if user_state.bound_nft_mint != Pubkey::default() {
        return Err(error!(BindingError::UserAlreadyBound));
    }

    // 6. Clear old owner's UserState
    // Only clear if old owner's bound_nft_mint still points to THIS NFT.
    // If old owner has already unbound (via unbind_nft) and rebound to a different NFT,
    // we must NOT overwrite their new binding.
    if old_owner_state.bound_nft_mint == nft_mint.key() {
        old_owner_state.bound_nft_mint = Pubkey::default();
    }

    // 7. Update new owner's UserState
    user_state.bound_nft_mint = nft_mint.key();
    user_state.bump = ctx.bumps.user_state;

    // 8. Update NftBindingState
    nft_binding_state.owner = user.key();
    nft_binding_state.last_bound_at = clock.unix_timestamp;
    // Note: released_amount is NOT reset, it follows the NFT

    msg!("NFT rebound successfully: mint={}, new_owner={}, old_owner={}",
        nft_mint.key(), user.key(), old_owner);

    Ok(())
}

/// Handler for claim_released_tokens instruction
pub fn handler_claim_released_tokens(
    ctx: Context<ClaimReleasedTokens>,
) -> Result<()> {
    let user = &ctx.accounts.user;
    let nft_mint = &ctx.accounts.nft_mint;
    let user_nft_token_account = &ctx.accounts.user_nft_token_account;
    let nft_binding_state = &mut ctx.accounts.nft_binding_state;
    let user_state = &ctx.accounts.user_state;
    let airdrop_vault = &mut ctx.accounts.airdrop_vault;
    let airdrop_vault_token_account = &ctx.accounts.airdrop_vault_token_account;
    let user_token_account = &ctx.accounts.user_token_account;
    let clock = Clock::get()?;

    // 1. Verify NFT ownership
    verify_nft_ownership(user_nft_token_account, &user.key(), &nft_mint.key())?;

    // 2. Verify binding is active (user_state matches current NFT)
    require!(
        user_state.bound_nft_mint == nft_mint.key(),
        TokenReleaseError::BindingNotActive
    );

    // 3. Verify binding state owner matches user
    require!(
        nft_binding_state.owner == user.key(),
        TokenReleaseError::BindingNotActive
    );

    // 4. Calculate releasable amount
    let (releasable_amount, _binding_days) = calculate_releasable_amount(
        nft_binding_state,
        clock.unix_timestamp
    )?;

    // 5. Check if there are tokens to release
    require!(
        releasable_amount > 0,
        TokenReleaseError::NoTokensToRelease
    );

    // 6. Check vault balance
    require!(
        airdrop_vault_token_account.amount >= releasable_amount,
        TokenReleaseError::InsufficientVaultBalance
    );

    // 7. Transfer tokens from airdrop vault to user using PDA signer
    let token_mint = airdrop_vault.token_mint.key();
    let vault_seeds = &[
        crate::constants::AIRDROP_VAULT_SEED,
        token_mint.as_ref(),
        &[airdrop_vault.bump],
    ];
    let signer_seeds = &[&vault_seeds[..]];

    let cpi_accounts = Transfer {
        from: airdrop_vault_token_account.to_account_info(),
        to: user_token_account.to_account_info(),
        authority: airdrop_vault.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    token::transfer(cpi_ctx, releasable_amount)?;

    // 8. Update binding state
    nft_binding_state.released_amount = nft_binding_state.released_amount
        .checked_add(releasable_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // 9. Update airdrop vault total_released
    airdrop_vault.total_released = airdrop_vault.total_released
        .checked_add(releasable_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // 9. Emit event
    emit!(TokensClaimedEvent {
        user: user.key(),
        nft_mint: nft_mint.key(),
        amount: releasable_amount,
        total_released: nft_binding_state.released_amount,
        timestamp: clock.unix_timestamp,
    });

    msg!("Tokens claimed successfully: user={}, nft={}, amount={}, total_released={}",
        user.key(), nft_mint.key(), releasable_amount, nft_binding_state.released_amount);

    Ok(())
}

/// Handler for query_releasable_tokens instruction
pub fn handler_query_releasable_tokens(
    ctx: Context<QueryReleasableTokens>,
) -> Result<TokenReleaseInfo> {
    let user = &ctx.accounts.user;
    let nft_mint = &ctx.accounts.nft_mint;
    let user_nft_token_account = &ctx.accounts.user_nft_token_account;
    let nft_binding_state = &ctx.accounts.nft_binding_state;
    let user_state = &ctx.accounts.user_state;
    let clock = Clock::get()?;

    // 1. Verify NFT ownership
    verify_nft_ownership(user_nft_token_account, &user.key(), &nft_mint.key())?;

    // 2. Verify binding is active
    require!(
        user_state.bound_nft_mint == nft_mint.key(),
        TokenReleaseError::BindingNotActive
    );

    require!(
        nft_binding_state.owner == user.key(),
        TokenReleaseError::BindingNotActive
    );

    // 3. Calculate releasable amount
    let (releasable_amount, binding_days) = calculate_releasable_amount(
        nft_binding_state,
        clock.unix_timestamp
    )?;

    // 4. Return query result
    Ok(TokenReleaseInfo {
        releasable_amount,
        total_released: nft_binding_state.released_amount,
        total_release: nft_binding_state.total_release,
        binding_days,
    })
}
