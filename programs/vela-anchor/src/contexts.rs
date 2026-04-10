use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{Metadata, MetadataAccount},
    token::{Mint, Token, TokenAccount},
};
use crate::structs::{DiamondCollectionState, GoldCollectionState, ReferralManager, ReferralStorage, WalletIdMapping, LockedTokenVault, AirdropVault, NftBindingState, UserState, GlobalState, UserStakeAccount};
use crate::errors::{LockedVaultError, NodeError, BindingError, ReferralError, StakeError};
use crate::constants::{NFT_BINDING_SEED, USER_STATE_SEED, LOCKED_VAULT_SEED, AIRDROP_VAULT_SEED, nft_authority_pubkey, dead_address_pubkey};

// ============================================================================
// NFT Contexts
// ============================================================================

/// CreateDiamondCollection account structure (Diamond Node Collection)
#[derive(Accounts)]
pub struct CreateDiamondCollection<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [b"collection_authority"],
        bump
    )]
    /// CHECK: VEL-08 legacy PDA, not used by any instruction logic; actual authority is payer (hardcoded admin address)
    pub collection_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = payer.key(),
        mint::freeze_authority = payer.key(),
    )]
    pub collection_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 32 + 8 + 8 + 1 + 1,
        seeds = [b"diamond_collection"],
        bump
    )]
    pub diamond_collection_state: Account<'info, DiamondCollectionState>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = collection_mint,
        associated_token::authority = payer,
    )]
    pub collection_token_account: Account<'info, TokenAccount>,

    /// CHECK: Metadata account
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,

    /// CHECK: Master Edition account
    #[account(mut)]
    pub collection_master_edition: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// CreateGoldCollection account structure (Gold Node Collection)
#[derive(Accounts)]
pub struct CreateGoldCollection<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [b"collection_authority"],
        bump
    )]
    /// CHECK: VEL-08 legacy PDA, not used by any instruction logic; actual authority is payer (hardcoded admin address)
    pub collection_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = payer.key(),
        mint::freeze_authority = payer.key(),
    )]
    pub collection_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 32 + 8 + 8 + 1 + 1,
        seeds = [b"gold_collection"],
        bump
    )]
    pub gold_collection_state: Account<'info, GoldCollectionState>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = collection_mint,
        associated_token::authority = payer,
    )]
    pub collection_token_account: Account<'info, TokenAccount>,

    /// CHECK: Metadata account
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,

    /// CHECK: Master Edition account
    #[account(mut)]
    pub collection_master_edition: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// VerifyDiamondOwnership account structure - verify Diamond Node ownership
#[derive(Accounts)]
pub struct VerifyDiamondOwnership<'info> {
    /// CHECK: user address to check
    pub user: UncheckedAccount<'info>,

    pub nft_mint: Account<'info, Mint>,

    pub user_token_account: Account<'info, TokenAccount>,

    /// CHECK: NFT Metadata account - validated via Metaplex PDA seeds
    #[account(
        seeds = [b"metadata", anchor_spl::metadata::mpl_token_metadata::ID.as_ref(), nft_mint.key().as_ref()],
        seeds::program = anchor_spl::metadata::mpl_token_metadata::ID,
        bump,
    )]
    pub nft_metadata: UncheckedAccount<'info>,

    /// CHECK: NFT Master Edition account - validated via Metaplex PDA seeds
    #[account(
        seeds = [b"metadata", anchor_spl::metadata::mpl_token_metadata::ID.as_ref(), nft_mint.key().as_ref(), b"edition"],
        seeds::program = anchor_spl::metadata::mpl_token_metadata::ID,
        bump,
    )]
    pub nft_master_edition: UncheckedAccount<'info>,

    pub collection_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

/// VerifyGoldOwnership account structure - verify Gold Node ownership
#[derive(Accounts)]
pub struct VerifyGoldOwnership<'info> {
    /// CHECK: user address to check
    pub user: UncheckedAccount<'info>,

    pub nft_mint: Account<'info, Mint>,

    pub user_token_account: Account<'info, TokenAccount>,

    /// CHECK: NFT Metadata account - validated via Metaplex PDA seeds
    #[account(
        seeds = [b"metadata", anchor_spl::metadata::mpl_token_metadata::ID.as_ref(), nft_mint.key().as_ref()],
        seeds::program = anchor_spl::metadata::mpl_token_metadata::ID,
        bump,
    )]
    pub nft_metadata: UncheckedAccount<'info>,

    /// CHECK: NFT Master Edition account - validated via Metaplex PDA seeds
    #[account(
        seeds = [b"metadata", anchor_spl::metadata::mpl_token_metadata::ID.as_ref(), nft_mint.key().as_ref(), b"edition"],
        seeds::program = anchor_spl::metadata::mpl_token_metadata::ID,
        bump,
    )]
    pub nft_master_edition: UncheckedAccount<'info>,

    pub collection_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

/// BatchMintDiamondNFT account structure - Diamond Node batch mint (update counter only)
#[derive(Accounts)]
pub struct BatchMintDiamondNFT<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub collection_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"diamond_collection"],
        bump = diamond_collection_state.bump,
    )]
    pub diamond_collection_state: Account<'info, DiamondCollectionState>,

    pub system_program: Program<'info, System>,
}

/// BatchMintGoldNFT account structure - Gold Node batch mint (update counter only)
#[derive(Accounts)]
pub struct BatchMintGoldNFT<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub collection_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"gold_collection"],
        bump = gold_collection_state.bump,
    )]
    pub gold_collection_state: Account<'info, GoldCollectionState>,

    pub system_program: Program<'info, System>,
}

/// MintDiamondNFT account structure - Diamond Node single mint
#[derive(Accounts)]
pub struct MintDiamondNFT<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        constraint = collection_mint.key() == diamond_collection_state.collection_mint @ NodeError::InvalidCollectionMint
    )]
    pub collection_mint: Account<'info, Mint>,

    /// CHECK: Collection Metadata account
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,

    /// CHECK: Collection Master Edition account
    pub collection_master_edition: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"diamond_collection"],
        bump = diamond_collection_state.bump,
    )]
    pub diamond_collection_state: Account<'info, DiamondCollectionState>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = payer.key(),
        mint::freeze_authority = payer.key(),
    )]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = payer,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    /// CHECK: Metadata account
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,

    /// CHECK: Master Edition account
    #[account(mut)]
    pub nft_master_edition: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// MintGoldNFT account structure - Gold Node single mint
#[derive(Accounts)]
pub struct MintGoldNFT<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        constraint = collection_mint.key() == gold_collection_state.collection_mint @ NodeError::InvalidCollectionMint
    )]
    pub collection_mint: Account<'info, Mint>,

    /// CHECK: Collection Metadata account
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,

    /// CHECK: Collection Master Edition account
    pub collection_master_edition: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"gold_collection"],
        bump = gold_collection_state.bump,
    )]
    pub gold_collection_state: Account<'info, GoldCollectionState>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = payer.key(),
        mint::freeze_authority = payer.key(),
    )]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = payer,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    /// CHECK: Metadata account
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,

    /// CHECK: Master Edition account
    #[account(mut)]
    pub nft_master_edition: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// ============================================================================
// Referral Contexts
// ============================================================================

/// Initialize ReferralManager and 9 ReferralStorage PDAs
#[derive(Accounts)]
#[instruction(root_wallet: Pubkey)]
pub struct InitializeReferralManager<'info> {
    #[account(
        mut,
        constraint = authority.key() == nft_authority_pubkey()
            @ ReferralError::Unauthorized
    )]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = ReferralManager::SIZE,
        seeds = [ReferralManager::SEED],
        bump
    )]
    pub manager: Account<'info, ReferralManager>,

    #[account(
        init,
        payer = authority,
        space = ReferralStorage::INIT_SPACE,
        seeds = [ReferralStorage::SEED_PREFIX, &[1u8]],
        bump
    )]
    pub storage_1: Account<'info, ReferralStorage>,

    #[account(
        init,
        payer = authority,
        space = ReferralStorage::INIT_SPACE,
        seeds = [ReferralStorage::SEED_PREFIX, &[2u8]],
        bump
    )]
    pub storage_2: Account<'info, ReferralStorage>,

    #[account(
        init,
        payer = authority,
        space = ReferralStorage::INIT_SPACE,
        seeds = [ReferralStorage::SEED_PREFIX, &[3u8]],
        bump
    )]
    pub storage_3: Account<'info, ReferralStorage>,

    #[account(
        init,
        payer = authority,
        space = ReferralStorage::INIT_SPACE,
        seeds = [ReferralStorage::SEED_PREFIX, &[4u8]],
        bump
    )]
    pub storage_4: Account<'info, ReferralStorage>,

    #[account(
        init,
        payer = authority,
        space = ReferralStorage::INIT_SPACE,
        seeds = [ReferralStorage::SEED_PREFIX, &[5u8]],
        bump
    )]
    pub storage_5: Account<'info, ReferralStorage>,

    #[account(
        init,
        payer = authority,
        space = ReferralStorage::INIT_SPACE,
        seeds = [ReferralStorage::SEED_PREFIX, &[6u8]],
        bump
    )]
    pub storage_6: Account<'info, ReferralStorage>,

    #[account(
        init,
        payer = authority,
        space = ReferralStorage::INIT_SPACE,
        seeds = [ReferralStorage::SEED_PREFIX, &[7u8]],
        bump
    )]
    pub storage_7: Account<'info, ReferralStorage>,

    #[account(
        init,
        payer = authority,
        space = ReferralStorage::INIT_SPACE,
        seeds = [ReferralStorage::SEED_PREFIX, &[8u8]],
        bump
    )]
    pub storage_8: Account<'info, ReferralStorage>,

    #[account(
        init,
        payer = authority,
        space = ReferralStorage::INIT_SPACE,
        seeds = [ReferralStorage::SEED_PREFIX, &[9u8]],
        bump
    )]
    pub storage_9: Account<'info, ReferralStorage>,

    /// Wallet ID mapping account for root user
    #[account(
        init,
        payer = authority,
        space = WalletIdMapping::SIZE,
        seeds = [WalletIdMapping::SEED_PREFIX, root_wallet.as_ref()],
        bump
    )]
    pub wallet_mapping: Account<'info, WalletIdMapping>,

    pub system_program: Program<'info, System>,
}

/// Add referrer (automatically select available storage)
#[derive(Accounts)]
#[instruction(wallet: Pubkey, parent_id: u32)]
pub struct AddReferral<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Wallet owner must sign to prove ownership, prevents frontrunning attacks
    /// mut: needed because registration fee is deducted from this account
    #[account(mut, constraint = wallet_signer.key() == wallet @ ReferralError::WalletOwnerMismatch)]
    pub wallet_signer: Signer<'info>,

    #[account(
        mut,
        seeds = [ReferralManager::SEED],
        bump
    )]
    pub manager: Account<'info, ReferralManager>,

    /// Explicitly pass all 9 storage PDAs (use UncheckedAccount for manual realloc)
    /// CHECK: verify PDA in function
    #[account(mut)]
    pub storage_1: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    #[account(mut)]
    pub storage_2: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    #[account(mut)]
    pub storage_3: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    #[account(mut)]
    pub storage_4: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    #[account(mut)]
    pub storage_5: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    #[account(mut)]
    pub storage_6: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    #[account(mut)]
    pub storage_7: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    #[account(mut)]
    pub storage_8: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    #[account(mut)]
    pub storage_9: UncheckedAccount<'info>,

    /// Wallet ID mapping account
    #[account(
        init,
        payer = payer,
        space = WalletIdMapping::SIZE,
        seeds = [WalletIdMapping::SEED_PREFIX, wallet.as_ref()],
        bump
    )]
    pub wallet_mapping: Account<'info, WalletIdMapping>,

    /// 全局状态（读取 referral_fee_wallet 地址）
    #[account(
        seeds = [GlobalState::SEED_PREFIX],
        bump = global_state.bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    /// 推荐人注册费收款钱包（必须与 global_state.referral_fee_wallet 一致）
    /// CHECK: 通过与 global_state.referral_fee_wallet 比较来校验
    #[account(
        mut,
        constraint = referral_fee_wallet.key() == global_state.referral_fee_wallet
            @ ReferralError::InvalidFeeWallet
    )]
    pub referral_fee_wallet: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

/// Query ID by wallet address
#[derive(Accounts)]
#[instruction(wallet: Pubkey)]
pub struct GetWalletId<'info> {
    /// Wallet ID mapping account
    /// CHECK: verify PDA in function
    pub wallet_mapping: UncheckedAccount<'info>,
}

/// Get referrer information
#[derive(Accounts)]
pub struct GetReferral<'info> {
    /// Explicitly pass all 9 storage PDAs (read-only)
    /// CHECK: verify PDA in function
    pub storage_1: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_2: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_3: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_4: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_5: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_6: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_7: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_8: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_9: UncheckedAccount<'info>,
}

/// Get wallet complete information (wallet -> referral_id -> ReferralData)
#[derive(Accounts)]
#[instruction(wallet: Pubkey)]
pub struct GetWalletInfo<'info> {
    /// Wallet ID mapping account
    /// CHECK: verify PDA in function
    pub wallet_mapping: UncheckedAccount<'info>,

    /// Explicitly pass all 9 storage PDAs (read-only)
    /// CHECK: verify PDA in function
    pub storage_1: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_2: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_3: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_4: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_5: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_6: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_7: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_8: UncheckedAccount<'info>,

    /// CHECK: verify PDA in function
    pub storage_9: UncheckedAccount<'info>,
}

// ============================================================================
// Locked Token Vault Contexts
// ============================================================================

#[derive(Accounts)]
pub struct LockTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [LOCKED_VAULT_SEED, locked_vault.token_mint.as_ref()],
        bump = locked_vault.bump
    )]
    pub locked_vault: Account<'info, LockedTokenVault>,

    #[account(
        mut,
        associated_token::mint = locked_vault.token_mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.key() == locked_vault.vault_token_account @ LockedVaultError::InvalidVaultTokenAccount,
        constraint = vault_token_account.mint == locked_vault.token_mint @ LockedVaultError::InvalidTokenMint,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

// ============================================================================
// NFT Binding Contexts
// ============================================================================

/// Context for bind_nft instruction
#[derive(Accounts)]
pub struct BindNft<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// NFT mint account
    pub nft_mint: Account<'info, Mint>,

    /// User's token account holding the NFT
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    /// NFT metadata account — PDA derived from nft_mint, verified by Metaplex program
    #[account(
        seeds = [
            b"metadata",
            anchor_spl::metadata::mpl_token_metadata::ID.as_ref(),
            nft_mint.key().as_ref(),
        ],
        seeds::program = anchor_spl::metadata::mpl_token_metadata::ID,
        bump,
    )]
    pub nft_metadata: Account<'info, MetadataAccount>,

    /// NFT binding state PDA (seeds: [NFT_BINDING_SEED, nft_mint])
    #[account(
        init,
        payer = user,
        space = NftBindingState::SIZE,
        seeds = [NFT_BINDING_SEED, nft_mint.key().as_ref()],
        bump
    )]
    pub nft_binding_state: Account<'info, NftBindingState>,

    /// User state PDA (seeds: [USER_STATE_SEED, user])
    #[account(
        init_if_needed,
        payer = user,
        space = UserState::SIZE,
        seeds = [USER_STATE_SEED, user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,

    /// Diamond collection state (for verification)
    #[account(
        seeds = [b"diamond_collection"],
        bump = diamond_collection_state.bump
    )]
    pub diamond_collection_state: Account<'info, DiamondCollectionState>,

    /// Gold collection state (for verification)
    #[account(
        seeds = [b"gold_collection"],
        bump = gold_collection_state.bump
    )]
    pub gold_collection_state: Account<'info, GoldCollectionState>,

    pub system_program: Program<'info, System>,
}

/// Context for verify_binding instruction
#[derive(Accounts)]
pub struct VerifyBinding<'info> {
    pub user: Signer<'info>,

    /// NFT mint account
    pub nft_mint: Account<'info, Mint>,

    /// User's token account holding the NFT
    pub user_token_account: Account<'info, TokenAccount>,

    /// CHECK: NFT binding state PDA - may not exist if NFT has never been bound.
    /// Validated manually in handler via PDA derivation and deserialization.
    pub nft_binding_state: UncheckedAccount<'info>,
}

/// Context for rebind_nft instruction
#[derive(Accounts)]
pub struct RebindNft<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// NFT mint account
    pub nft_mint: Account<'info, Mint>,

    /// New owner's token account holding the NFT
    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// NFT binding state PDA (must exist)
    #[account(
        mut,
        seeds = [NFT_BINDING_SEED, nft_mint.key().as_ref()],
        bump = nft_binding_state.bump
    )]
    pub nft_binding_state: Account<'info, NftBindingState>,

    /// New owner's user state PDA
    #[account(
        init_if_needed,
        payer = user,
        space = UserState::SIZE,
        seeds = [USER_STATE_SEED, user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,

    /// Old owner's user state PDA
    #[account(
        mut,
        seeds = [USER_STATE_SEED, nft_binding_state.owner.as_ref()],
        bump = old_owner_state.bump
    )]
    pub old_owner_state: Account<'info, UserState>,

    pub system_program: Program<'info, System>,
}

/// Context for unbind_nft instruction
#[derive(Accounts)]
pub struct UnbindNft<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// User state PDA
    #[account(
        mut,
        seeds = [USER_STATE_SEED, user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,

    /// NFT binding state PDA (verified via seeds derived from user_state.bound_nft_mint)
    #[account(
        mut,
        seeds = [NFT_BINDING_SEED, user_state.bound_nft_mint.as_ref()],
        bump = nft_binding_state.bump
    )]
    pub nft_binding_state: Account<'info, NftBindingState>,

    /// NFT mint account (must match bound mint)
    #[account(
        address = user_state.bound_nft_mint
    )]
    pub nft_mint: Account<'info, Mint>,

    /// User's ATA for the bound NFT mint (deterministic, prevents bypass with alternate accounts)
    #[account(
        associated_token::mint = nft_mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

/// Context for claim_released_tokens instruction
#[derive(Accounts)]
pub struct ClaimReleasedTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// NFT mint account
    pub nft_mint: Account<'info, Mint>,

    /// User's token account holding the NFT
    pub user_nft_token_account: Account<'info, TokenAccount>,

    /// User's token account for receiving released tokens
    #[account(
        mut,
        associated_token::mint = airdrop_vault.token_mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// NFT binding state PDA
    #[account(
        mut,
        seeds = [NFT_BINDING_SEED, nft_mint.key().as_ref()],
        bump = nft_binding_state.bump
    )]
    pub nft_binding_state: Account<'info, NftBindingState>,

    /// User state PDA
    #[account(
        seeds = [USER_STATE_SEED, user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,

    /// Airdrop vault PDA (replaces locked_vault for NFT airdrops)
    #[account(
        mut,
        seeds = [AIRDROP_VAULT_SEED, airdrop_vault.token_mint.as_ref()],
        bump = airdrop_vault.bump
    )]
    pub airdrop_vault: Account<'info, AirdropVault>,

    /// Airdrop vault token account
    #[account(
        mut,
        constraint = airdrop_vault_token_account.key() == airdrop_vault.vault_token_account @ LockedVaultError::InvalidVaultTokenAccount
    )]
    pub airdrop_vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

/// Context for query_releasable_tokens instruction
#[derive(Accounts)]
pub struct QueryReleasableTokens<'info> {
    pub user: Signer<'info>,

    /// NFT mint account
    pub nft_mint: Account<'info, Mint>,

    /// User's token account holding the NFT
    pub user_nft_token_account: Account<'info, TokenAccount>,

    /// NFT binding state PDA
    #[account(
        seeds = [NFT_BINDING_SEED, nft_mint.key().as_ref()],
        bump = nft_binding_state.bump
    )]
    pub nft_binding_state: Account<'info, NftBindingState>,

    /// User state PDA
    #[account(
        seeds = [USER_STATE_SEED, user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
}

// ============================================================================
// Staking Contexts
// ============================================================================

/// Create staking order context
#[derive(Accounts)]
pub struct CreateStake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// Box is used to move this large account to heap instead of stack
    #[account(
        init_if_needed,
        payer = user,
        space = crate::structs::UserStakeAccount::SIZE,
        seeds = [b"user_stake", user.key().as_ref()],
        bump,
        constraint = user_stake_account.owner == Pubkey::default() || user_stake_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub user_stake_account: Box<Account<'info, crate::structs::UserStakeAccount>>,

    /// Box is used to reduce stack usage
    #[account(
        mut,
        seeds = [GlobalState::SEED_PREFIX],
        bump = global_state.bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ StakeError::Unauthorized,
        constraint = user_token_account.mint == global_state.stake_token_mint @ StakeError::TokenMintMismatch
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    /// locked_vault PDA (for validation)
    /// CHECK: Seeds validation moved to handler to reduce stack usage
    #[account(mut)]
    pub locked_vault: Box<Account<'info, LockedTokenVault>>,

    /// Vault token account (receives staked principal)
    #[account(
        mut,
        constraint = vault_token_account.key() == locked_vault.vault_token_account @ StakeError::InvalidLockedVault
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    // Referral system related accounts
    /// User wallet ID mapping (must exist, proves user has bound referrer)
    #[account(
        seeds = [b"wallet_id_mapping", user.key().as_ref()],
        bump
    )]
    pub wallet_mapping: Box<Account<'info, WalletIdMapping>>,

    /// CHECK: Referral storage PDA 1
    #[account(mut)]
    pub storage_1: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 2
    #[account(mut)]
    pub storage_2: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 3
    #[account(mut)]
    pub storage_3: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 4
    #[account(mut)]
    pub storage_4: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 5
    #[account(mut)]
    pub storage_5: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 6
    #[account(mut)]
    pub storage_6: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 7
    #[account(mut)]
    pub storage_7: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 8
    #[account(mut)]
    pub storage_8: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 9
    #[account(mut)]
    pub storage_9: UncheckedAccount<'info>,

    /// User state account (optional, used to query bound NFT)
    /// Note: Seeds constraint removed because it causes access violations when Option is None
    pub user_state: Option<Box<Account<'info, UserState>>>,

    /// NFT binding state account (optional, used to query node type)
    /// Note: Seeds constraint removed because it causes access violations when Option is None
    pub nft_binding_state: Option<Box<Account<'info, NftBindingState>>>,

    /// User's NFT token account (optional, used to verify NFT ownership)
    /// Note: Constraints removed because they cause access violations when Option is None
    pub user_nft_account: Option<Box<Account<'info, TokenAccount>>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Claim interest (claim accumulated interest for a staking order)
#[derive(Accounts)]
#[instruction(order_index: u8)]
pub struct ClaimInterest<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user_stake", user.key().as_ref()],
        bump = user_stake_account.bump,
        constraint = user_stake_account.owner == user.key()
    )]
    pub user_stake_account: Box<Account<'info, UserStakeAccount>>,

    #[account(
        mut,
        seeds = [GlobalState::SEED_PREFIX],
        bump = global_state.bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    /// User token account (receives interest)
    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ StakeError::Unauthorized,
        constraint = user_token_account.mint == global_state.stake_token_mint @ StakeError::TokenMintMismatch
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    /// locked_vault PDA (used for signing)
    #[account(mut)]
    pub locked_vault: Box<Account<'info, LockedTokenVault>>,

    /// Token pool account (pays interest)
    #[account(
        mut,
        constraint = vault_token_account.key() == locked_vault.vault_token_account @ StakeError::InvalidLockedVault
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    /// Dead address token account (receives interest tax)
    #[account(
        mut,
        constraint = dead_address_token_account.owner == dead_address_pubkey() @ StakeError::InvalidDeadAddress,
        constraint = dead_address_token_account.mint == global_state.stake_token_mint @ StakeError::TokenMintMismatch
    )]
    pub dead_address_token_account: Box<Account<'info, TokenAccount>>,

    // Referral system related accounts (used for community reward distribution)
    /// User wallet ID mapping (must exist)
    #[account(
        seeds = [b"wallet_id_mapping", user.key().as_ref()],
        bump
    )]
    pub wallet_mapping: Box<Account<'info, WalletIdMapping>>,

    /// CHECK: Referral storage PDA 1
    #[account(mut)]
    pub storage_1: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 2
    #[account(mut)]
    pub storage_2: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 3
    #[account(mut)]
    pub storage_3: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 4
    #[account(mut)]
    pub storage_4: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 5
    #[account(mut)]
    pub storage_5: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 6
    #[account(mut)]
    pub storage_6: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 7
    #[account(mut)]
    pub storage_7: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 8
    #[account(mut)]
    pub storage_8: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 9
    #[account(mut)]
    pub storage_9: UncheckedAccount<'info>,

    /// User state account (optional, used to query bound NFT)
    /// Note: Seeds constraint removed because it causes access violations when Option is None
    pub user_state: Option<Box<Account<'info, UserState>>>,

    /// NFT binding state account (optional, used to query node type)
    /// Note: Seeds constraint removed because it causes access violations when Option is None
    pub nft_binding_state: Option<Box<Account<'info, NftBindingState>>>,

    /// User's NFT token account (optional, used to verify NFT ownership)
    /// Note: Constraints removed because they cause access violations when Option is None
    pub user_nft_account: Option<Box<Account<'info, TokenAccount>>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Query pending interest (read-only, used with simulateTransaction)
#[derive(Accounts)]
#[instruction(order_index: u8)]
pub struct QueryPendingInterest<'info> {
    pub user: Signer<'info>,

    #[account(
        seeds = [b"user_stake", user.key().as_ref()],
        bump = user_stake_account.bump,
        constraint = user_stake_account.owner == user.key()
    )]
    pub user_stake_account: Box<Account<'info, UserStakeAccount>>,

    /// User state account (optional, used to query bound NFT)
    /// Note: Seeds constraint removed because it causes access violations when Option is None
    pub user_state: Option<Box<Account<'info, UserState>>>,

    /// NFT binding state account (optional, used to query node type)
    /// Note: Seeds constraint removed because it causes access violations when Option is None
    pub nft_binding_state: Option<Box<Account<'info, NftBindingState>>>,

    /// User's NFT token account (optional, used to verify NFT ownership)
    /// Note: Constraints removed because they cause access violations when Option is None
    pub user_nft_account: Option<Box<Account<'info, TokenAccount>>>,
}
/// Unstake (redeem principal + remaining interest) context
#[derive(Accounts)]
#[instruction(order_index: u8)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user_stake", user.key().as_ref()],
        bump = user_stake_account.bump,
        constraint = user_stake_account.owner == user.key()
    )]
    pub user_stake_account: Box<Account<'info, crate::structs::UserStakeAccount>>,

    #[account(
        mut,
        seeds = [GlobalState::SEED_PREFIX],
        bump = global_state.bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    /// User token account (receives principal + interest)
    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ StakeError::Unauthorized,
        constraint = user_token_account.mint == global_state.stake_token_mint @ StakeError::TokenMintMismatch
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    /// locked_vault PDA (used as signing authority)
    #[account(mut)]
    pub locked_vault: Box<Account<'info, LockedTokenVault>>,

    /// Vault token account (source of principal and interest)
    #[account(
        mut,
        constraint = vault_token_account.key() == locked_vault.vault_token_account @ StakeError::InvalidLockedVault
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    /// Dead address token account (receives interest tax)
    #[account(
        mut,
        constraint = dead_address_token_account.owner == dead_address_pubkey() @ StakeError::InvalidDeadAddress,
        constraint = dead_address_token_account.mint == global_state.stake_token_mint @ StakeError::TokenMintMismatch
    )]
    pub dead_address_token_account: Box<Account<'info, TokenAccount>>,

    // Referral system related accounts
    /// User wallet ID mapping (must exist)
    #[account(
        seeds = [b"wallet_id_mapping", user.key().as_ref()],
        bump
    )]
    pub wallet_mapping: Box<Account<'info, WalletIdMapping>>,

    /// CHECK: Referral storage PDA 1
    #[account(mut)]
    pub storage_1: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 2
    #[account(mut)]
    pub storage_2: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 3
    #[account(mut)]
    pub storage_3: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 4
    #[account(mut)]
    pub storage_4: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 5
    #[account(mut)]
    pub storage_5: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 6
    #[account(mut)]
    pub storage_6: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 7
    #[account(mut)]
    pub storage_7: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 8
    #[account(mut)]
    pub storage_8: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 9
    #[account(mut)]
    pub storage_9: UncheckedAccount<'info>,

    /// User state account (optional, used to query bound NFT)
    /// Note: Seeds constraint removed because it causes access violations when Option is None
    pub user_state: Option<Box<Account<'info, UserState>>>,

    /// NFT binding state account (optional, used to query node type)
    /// Note: Seeds constraint removed because it causes access violations when Option is None
    pub nft_binding_state: Option<Box<Account<'info, NftBindingState>>>,

    /// User's NFT token account (optional, used to verify NFT ownership)
    /// Note: Constraints removed because they cause access violations when Option is None
    pub user_nft_account: Option<Box<Account<'info, TokenAccount>>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// ============================================================================
// Community Reward Contexts
// ============================================================================

/// Query community status (read-only, used with .view())
#[derive(Accounts)]
pub struct QueryCommunityStatus<'info> {
    pub user: Signer<'info>,

    /// User wallet ID mapping (must exist)
    #[account(
        seeds = [b"wallet_id_mapping", user.key().as_ref()],
        bump
    )]
    pub wallet_mapping: Account<'info, WalletIdMapping>,

    /// CHECK: Referral storage PDA 1
    pub storage_1: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 2
    pub storage_2: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 3
    pub storage_3: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 4
    pub storage_4: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 5
    pub storage_5: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 6
    pub storage_6: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 7
    pub storage_7: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 8
    pub storage_8: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 9
    pub storage_9: UncheckedAccount<'info>,
}

/// Claim community profit (transfer community_profit tokens to user)
#[derive(Accounts)]
pub struct ClaimCommunityProfit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// User wallet ID mapping (must exist)
    #[account(
        seeds = [b"wallet_id_mapping", user.key().as_ref()],
        bump
    )]
    pub wallet_mapping: Account<'info, WalletIdMapping>,

    /// CHECK: Referral storage PDA 1
    #[account(mut)]
    pub storage_1: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 2
    #[account(mut)]
    pub storage_2: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 3
    #[account(mut)]
    pub storage_3: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 4
    #[account(mut)]
    pub storage_4: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 5
    #[account(mut)]
    pub storage_5: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 6
    #[account(mut)]
    pub storage_6: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 7
    #[account(mut)]
    pub storage_7: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 8
    #[account(mut)]
    pub storage_8: UncheckedAccount<'info>,
    /// CHECK: Referral storage PDA 9
    #[account(mut)]
    pub storage_9: UncheckedAccount<'info>,

    #[account(
        seeds = [GlobalState::SEED_PREFIX],
        bump = global_state.bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    /// locked_vault PDA (used for signing)
    #[account(
        mut,
        seeds = [LOCKED_VAULT_SEED, locked_vault.token_mint.as_ref()],
        bump = locked_vault.bump
    )]
    pub locked_vault: Box<Account<'info, LockedTokenVault>>,

    /// Token pool account (pays community rewards)
    #[account(
        mut,
        constraint = vault_token_account.key() == locked_vault.vault_token_account @ LockedVaultError::InvalidVaultTokenAccount
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// User token account (receives community rewards)
    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ StakeError::Unauthorized,
        constraint = user_token_account.mint == global_state.stake_token_mint @ StakeError::TokenMintMismatch
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// Dead address token account (receives tax)
    #[account(
        mut,
        constraint = dead_address_token_account.owner == dead_address_pubkey() @ StakeError::InvalidDeadAddress,
        constraint = dead_address_token_account.mint == global_state.stake_token_mint @ StakeError::TokenMintMismatch
    )]
    pub dead_address_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// ============================================================================
// Query Current Rates Context
// ============================================================================

/// Query current three-tier staking interest rates (read-only)
#[derive(Accounts)]
pub struct QueryCurrentRates<'info> {
    #[account(
        seeds = [GlobalState::SEED_PREFIX],
        bump = global_state.bump,
    )]
    pub global_state: Account<'info, GlobalState>,
}

// ============================================================================
// Node Pool Contexts
// ============================================================================

/// Claim node pool rewards
#[derive(Accounts)]
pub struct ClaimNodePoolReward<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [GlobalState::SEED_PREFIX],
        bump = global_state.bump,
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    /// NFT mint (used for NftBindingState PDA derivation)
    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [NFT_BINDING_SEED, nft_mint.key().as_ref()],
        bump = nft_binding_state.bump,
        constraint = nft_binding_state.owner == user.key() @ BindingError::NotNftOwner,
    )]
    pub nft_binding_state: Account<'info, NftBindingState>,

    /// User's NFT token account (verifies still holding the NFT)
    #[account(
        constraint = user_nft_account.owner == user.key() @ BindingError::NotNftOwner,
        constraint = user_nft_account.mint == nft_mint.key() @ BindingError::NotNftOwner,
        constraint = user_nft_account.amount >= 1 @ BindingError::NotNftOwner,
    )]
    pub user_nft_account: Account<'info, TokenAccount>,

    /// locked_vault PDA (used for signing transfers)
    #[account(
        mut,
        seeds = [LOCKED_VAULT_SEED, locked_vault.token_mint.as_ref()],
        bump = locked_vault.bump
    )]
    pub locked_vault: Box<Account<'info, LockedTokenVault>>,

    /// Vault token account (outgoing funds)
    #[account(
        mut,
        constraint = vault_token_account.key() == locked_vault.vault_token_account @ LockedVaultError::InvalidVaultTokenAccount
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// User's stake token account (receives payment)
    #[account(
        mut,
        associated_token::mint = locked_vault.token_mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// Dead address token account (receives tax)
    #[account(
        mut,
        constraint = dead_address_token_account.owner == dead_address_pubkey() @ StakeError::InvalidDeadAddress,
        constraint = dead_address_token_account.mint == global_state.stake_token_mint @ StakeError::TokenMintMismatch
    )]
    pub dead_address_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    // === Storage account needed for cross-epoch refresh ===
    /// CHECK: ReferralStorage PDA index=1 (contains root referrer, used to write unclaimed rewards)
    #[account(mut)]
    pub referral_storage_1: AccountInfo<'info>,
}

/// Query node pool status (read-only)
#[derive(Accounts)]
pub struct QueryNodePoolStatus<'info> {
    #[account(
        seeds = [GlobalState::SEED_PREFIX],
        bump = global_state.bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    /// Optional: user's NftBindingState (query personal claim status)
    pub nft_binding_state: Option<Account<'info, NftBindingState>>,
}

/// Query node pool reward amount for a specific user (read-only)
#[derive(Accounts)]
pub struct QueryNodePoolReward<'info> {
    /// User (for ownership verification)
    /// CHECK: This is safe because we verify ownership through constraints below
    pub user: AccountInfo<'info>,

    #[account(
        seeds = [GlobalState::SEED_PREFIX],
        bump = global_state.bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    /// NFT mint (used for NftBindingState PDA derivation)
    pub nft_mint: Account<'info, Mint>,

    /// User's NftBindingState (to determine node type and claim status)
    #[account(
        seeds = [NFT_BINDING_SEED, nft_mint.key().as_ref()],
        bump = nft_binding_state.bump,
        constraint = nft_binding_state.owner == user.key() @ BindingError::NotNftOwner,
    )]
    pub nft_binding_state: Account<'info, NftBindingState>,

    /// User's NFT token account (verifies still holding the NFT)
    #[account(
        constraint = user_nft_account.owner == user.key() @ BindingError::NotNftOwner,
        constraint = user_nft_account.mint == nft_mint.key() @ BindingError::NotNftOwner,
        constraint = user_nft_account.amount >= 1 @ BindingError::NotNftOwner,
    )]
    pub user_nft_account: Account<'info, TokenAccount>,
}

// ============================================================================
// Airdrop Fund Contexts
// ============================================================================

/// 存入空投基金（任何人都可以存入，无权限限制）
#[derive(Accounts)]
pub struct DepositAirdropFund<'info> {
    /// 存入者（任何人都可以，无权限限制）
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        mut,
        seeds = [AIRDROP_VAULT_SEED, airdrop_vault.token_mint.as_ref()],
        bump = airdrop_vault.bump
    )]
    pub airdrop_vault: Account<'info, AirdropVault>,

    /// 存入者的 token 账户
    #[account(
        mut,
        constraint = depositor_token_account.mint == airdrop_vault.token_mint
            @ LockedVaultError::InvalidTokenMint,
        constraint = depositor_token_account.owner == depositor.key(),
    )]
    pub depositor_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = airdrop_vault_token_account.key() == airdrop_vault.vault_token_account
            @ LockedVaultError::InvalidVaultTokenAccount,
    )]
    pub airdrop_vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
