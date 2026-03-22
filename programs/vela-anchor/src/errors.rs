use anchor_lang::prelude::*;

// ============================================================================
// Referral error definitions
// ============================================================================

#[error_code]
pub enum ReferralError {
    #[msg("All referral storage PDAs are full")]
    StorageFull,

    #[msg("Invalid PDA index, must be 1-9")]
    InvalidPdaIndex,

    #[msg("Invalid referral ID")]
    InvalidReferralId,

    #[msg("Parent referral not found")]
    ParentNotFound,

    #[msg("Referral slot not found")]
    SlotNotFound,

    #[msg("Invalid ID format")]
    InvalidId,

    #[msg("Required storage account not provided")]
    StorageNotProvided,

    #[msg("Referral not found")]
    ReferralNotFound,

    #[msg("All storage PDAs are full")]
    AllStoragesFull,

    #[msg("Root node already exists, only the first user can have parent_id=0")]
    RootNodeAlreadyExists,

    #[msg("Invalid index: out of bounds")]
    InvalidIndex,

    #[msg("Invalid data: corrupted or malformed")]
    InvalidData,

    #[msg("Circular reference detected in referral chain")]
    CircularReference,

    #[msg("Invalid referral storage PDA address")]
    InvalidStoragePDA,

    #[msg("Unauthorized: caller is not the authorized admin")]
    Unauthorized,

    #[msg("Wallet signer does not match the wallet parameter")]
    WalletOwnerMismatch,

    #[msg("Invalid referral fee wallet address")]
    InvalidFeeWallet,
}

// ============================================================================
// NFT error definitions
// ============================================================================

#[error_code]
pub enum NodeError {
    #[msg("Unauthorized: only admins can create Node Collection")]
    UnauthorizedAdmin,
    #[msg("Unauthorized: only primary address can mint Node NFT")]
    UnauthorizedMinter,
    #[msg("Max supply reached: cannot mint more Node NFT")]
    MaxSupplyReached,
    #[msg("Invalid collection mint: does not match state")]
    InvalidCollectionMint,
    #[msg("Invalid max supply: cannot be less than minted count")]
    InvalidMaxSupply,
}

// ============================================================================
// Locked Token Vault error definitions
// ============================================================================

#[error_code]
pub enum LockedVaultError {
    #[msg("Invalid token mint: must be the specified reward token")]
    InvalidTokenMint,
    #[msg("Invalid amount: must be greater than 0")]
    InvalidAmount,
    #[msg("Unauthorized: only authority can initialize vault")]
    UnauthorizedAuthority,
    #[msg("Invalid vault token account: does not match vault's token account")]
    InvalidVaultTokenAccount,
}

// ============================================================================
// NFT Binding error definitions
// ============================================================================

#[error_code]
pub enum BindingError {
    #[msg("NFT not owned by user")]
    NotNftOwner,
    #[msg("NFT does not belong to any valid node collection")]
    InvalidNodeCollection,
    #[msg("NFT already bound")]
    AlreadyBound,
    #[msg("Binding not found or invalid")]
    BindingNotFound,
    #[msg("NFT has been transferred, binding invalid")]
    NftTransferred,
    #[msg("Binding cooldown: must wait 15 days between bindings")]
    BindingCooldown,
    #[msg("Current owner still holds the NFT")]
    OwnerStillHoldsNft,
    #[msg("User already has an active NFT binding")]
    UserAlreadyBound,
    #[msg("Invalid PDA address")]
    InvalidPdaAddress,
}

// ============================================================================
// Token Release error definitions
// ============================================================================

#[error_code]
pub enum TokenReleaseError {
    #[msg("No tokens available for release")]
    NoTokensToRelease,
    #[msg("Vault balance insufficient")]
    InsufficientVaultBalance,
    #[msg("NFT binding not active")]
    BindingNotActive,
}

// ============================================================================
// Staking error definitions
// ============================================================================

#[error_code]
pub enum StakeError {
    #[msg("Invalid stake period type")]
    InvalidPeriodType,
    #[msg("Invalid stake amount (must be between 1,000 and 50,000 VELA)")]
    InvalidAmount,
    #[msg("Stake amount must be a whole number (no decimals)")]
    AmountMustBeWholeNumber,
    #[msg("Maximum stake orders reached (20)")]
    MaxStakesReached,
    #[msg("Order not found or invalid index")]
    InvalidOrderIndex,
    #[msg("Order is not active")]
    OrderNotActive,
    #[msg("Stake period not ended yet")]
    PeriodNotEnded,
    #[msg("No interest to claim")]
    NoInterestToClaim,
    #[msg("NFT binding state mismatch")]
    NftBindingMismatch,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Token mint mismatch")]
    TokenMintMismatch,
    #[msg("User not authorized")]
    Unauthorized,
    #[msg("User wallet mapping not found - must bind referrer first")]
    WalletMappingNotFound,
    #[msg("User not in referral system - must bind referrer before staking")]
    UserNotInReferralSystem,
    #[msg("Invalid locked vault PDA")]
    InvalidLockedVault,
    #[msg("Daily deposit cap exhausted")]
    DailyDepositCapExhausted,
    #[msg("Amount exceeds daily deposit cap remaining")]
    DailyDepositCapExceeded,
    #[msg("User total staked exceeds per-address cap")]
    UserStakeCapExceeded,
    #[msg("Claim too frequent: must wait at least 1 hour between claims")]
    ClaimTooFrequent,
    #[msg("Invalid dead address token account")]
    InvalidDeadAddress,
    #[msg("NFT ownership mismatch: user does not hold the NFT")]
    NftOwnershipMismatch,
}

// ============================================================================
// Node Pool error definitions
// ============================================================================

#[error_code]
pub enum NodePoolError {
    #[msg("No node pool rewards available")]
    NoPoolRewards,

    #[msg("Already claimed node pool reward this week")]
    AlreadyClaimedThisWeek,

    #[msg("No previous week data available yet")]
    NoPreviousWeekData,

    #[msg("Insufficient vault balance for transfer")]
    InsufficientVaultBalance,

    #[msg("Pool claimed count exceeded maximum shares")]
    ClaimedCountExceeded,
}
