use anchor_lang::prelude::*;

// Import all modules
pub mod constants;
pub mod errors;
pub mod structs;
pub mod macros;
pub mod contexts;
pub mod nft;
pub mod referral;
pub mod referral_utils;
pub mod events;
pub mod zero_copy_storage;
pub mod locked_vault;
pub mod nft_binding;
pub mod global;
pub mod stake_token;
pub mod community_reward;
pub mod node_pool;

// Use imports
use contexts::*;
use structs::*;
use global::*;

declare_id!("7dcorXkE1kJJxvG8rQCLwnFnmdP9BxgGpts31aMAADam");


#[program]
pub mod vela_anchor {
    use super::*;

    // ========== Diamond Node NFT ==========

    /// Create Diamond Node Collection
    pub fn create_diamond_collection(
        ctx: Context<CreateDiamondCollection>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        nft::create_diamond_collection(ctx, name, symbol, uri)
    }

    /// Mint Diamond Node NFT (single)
    pub fn mint_diamond_nft(
        ctx: Context<MintDiamondNFT>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        nft::mint_diamond_nft(ctx, name, symbol, uri)
    }

    /// Verify Diamond Node NFT ownership
    pub fn verify_diamond_ownership(
        ctx: Context<VerifyDiamondOwnership>,
    ) -> Result<DiamondOwnershipResult> {
        nft::verify_diamond_ownership(ctx)
    }

    // ========== Gold Node NFT ==========

    /// Create Gold Node Collection
    pub fn create_gold_collection(
        ctx: Context<CreateGoldCollection>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        nft::create_gold_collection(ctx, name, symbol, uri)
    }

    /// Mint Gold Node NFT (single)
    pub fn mint_gold_nft(
        ctx: Context<MintGoldNFT>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        nft::mint_gold_nft(ctx, name, symbol, uri)
    }

    /// Verify Gold Node NFT ownership
    pub fn verify_gold_ownership(
        ctx: Context<VerifyGoldOwnership>,
    ) -> Result<GoldOwnershipResult> {
        nft::verify_gold_ownership(ctx)
    }

    // ========== Referral System ==========

    /// Initialize ReferralManager and 9 ReferralStorage PDAs with root user
    pub fn initialize_referral_manager(
        ctx: Context<InitializeReferralManager>,
        root_wallet: Pubkey,
    ) -> Result<()> {
        referral::handler_initialize(ctx, root_wallet)
    }

    /// Add referrer (automatically select available storage)
    pub fn add_referral(
        ctx: Context<AddReferral>,
        wallet: Pubkey,
        parent_id: u32,
    ) -> Result<u32> {
        referral::handler_add_referral(ctx, wallet, parent_id)
    }

    /// Get referrer information
    pub fn get_referral(
        ctx: Context<GetReferral>,
        referral_id: u32,
    ) -> Result<ReferralData> {
        referral::handler_get_referral(ctx, referral_id)
    }

    /// Get referrer ID by wallet address
    pub fn get_wallet_id(
        ctx: Context<GetWalletId>,
        wallet: Pubkey,
    ) -> Result<Option<u32>> {
        referral::handler_get_wallet_id(ctx, wallet)
    }

    /// Get complete wallet information (wallet -> referral_id -> WalletInfoResult with parent_wallet)
    pub fn get_wallet_info(
        ctx: Context<GetWalletInfo>,
        wallet: Pubkey,
    ) -> Result<Option<WalletInfoResult>> {
        referral::handler_get_wallet_info(ctx, wallet)
    }

    // ========== Global Initialization ==========

    /// Initialize global state (combines LockedTokenVault + GlobalState)
    pub fn initialize_global(
        ctx: Context<InitializeGlobal>,
    ) -> Result<()> {
        global::handler_initialize_global(ctx)
    }

    // ========== Locked Token Vault ==========

    /// Lock tokens into the vault
    pub fn lock_tokens(
        ctx: Context<LockTokens>,
        amount: u64,
    ) -> Result<()> {
        locked_vault::handler_lock_tokens(ctx, amount)
    }

    /// 存入空投基金（任何人都可以存入）
    pub fn deposit_airdrop_fund(
        ctx: Context<DepositAirdropFund>,
        amount: u64,
    ) -> Result<()> {
        locked_vault::handler_deposit_airdrop_fund(ctx, amount)
    }

    // ========== NFT Binding ==========

    /// Bind NFT to user account
    pub fn bind_nft(
        ctx: Context<BindNft>,
    ) -> Result<()> {
        nft_binding::handler_bind_nft(ctx)
    }

    /// Verify NFT binding status
    pub fn verify_binding(
        ctx: Context<VerifyBinding>,
    ) -> Result<BindingVerificationResult> {
        nft_binding::handler_verify_binding(ctx)
    }

    /// Unbind NFT (when user no longer holds the NFT)
    pub fn unbind_nft(
        ctx: Context<UnbindNft>,
    ) -> Result<()> {
        nft_binding::handler_unbind_nft(ctx)
    }

    /// Rebind NFT to new owner
    pub fn rebind_nft(
        ctx: Context<RebindNft>,
    ) -> Result<()> {
        nft_binding::handler_rebind_nft(ctx)
    }

    /// Claim released tokens from bound NFT
    pub fn claim_released_tokens(
        ctx: Context<ClaimReleasedTokens>,
    ) -> Result<()> {
        nft_binding::handler_claim_released_tokens(ctx)
    }

    /// Query releasable token amount for bound NFT
    pub fn query_releasable_tokens(
        ctx: Context<QueryReleasableTokens>,
    ) -> Result<TokenReleaseInfo> {
        nft_binding::handler_query_releasable_tokens(ctx)
    }

    // ========== Staking Functions ==========

    /// Create a new staking order
    pub fn create_stake(
        ctx: Context<CreateStake>,
        amount: u64,
        period_type: u8,
    ) -> Result<()> {
        stake_token::handler_create_stake(ctx, amount, period_type)
    }

    /// Unstake: redeem principal and remaining interest after period ends
    pub fn unstake(
        ctx: Context<Unstake>,
        order_index: u8,
    ) -> Result<()> {
        stake_token::handler_unstake(ctx, order_index)
    }

    /// Claim interest: claim accumulated interest for a specific order (including NFT bonus and tax)
    pub fn claim_interest(
        ctx: Context<ClaimInterest>,
        order_index: u8,
    ) -> Result<()> {
        stake_token::handler_claim_interest(ctx, order_index)
    }

    /// Query pending interest: query the expected pending interest for a specific order (read-only)
    pub fn query_pending_interest(
        ctx: Context<QueryPendingInterest>,
        order_index: u8,
    ) -> Result<PendingInterestResult> {
        stake_token::handler_query_pending_interest(ctx, order_index)
    }

    // ========== Community Reward Functions ==========

    /// Query community status: query user's community status and level (read-only)
    pub fn query_community_status(
        ctx: Context<QueryCommunityStatus>,
    ) -> Result<CommunityStatusResult> {
        stake_token::handler_query_community_status(ctx)
    }

    /// Claim community profit: claim community rewards
    pub fn claim_community_profit(
        ctx: Context<ClaimCommunityProfit>,
    ) -> Result<()> {
        stake_token::handler_claim_community_profit(ctx)
    }

    /// Query current lock-up interest rates for all three tiers (read-only, affected by halving mechanism)
    pub fn query_current_rates(
        ctx: Context<QueryCurrentRates>,
    ) -> Result<CurrentRatesResult> {
        stake_token::handler_query_current_rates(ctx)
    }

    // ========== Node Pool Functions ==========

    /// Claim node pool reward (for Diamond/Gold node users)
    pub fn claim_node_pool_reward(
        ctx: Context<ClaimNodePoolReward>,
    ) -> Result<()> {
        node_pool::handler_claim_node_pool_reward(ctx)
    }

    /// Query node pool status (read-only)
    pub fn query_node_pool_status(
        ctx: Context<QueryNodePoolStatus>,
    ) -> Result<NodePoolStatusResult> {
        node_pool::handler_query_node_pool_status(ctx)
    }

    /// Query node pool reward amount for a specific user (read-only)
    pub fn query_node_pool_reward(
        ctx: Context<QueryNodePoolReward>,
    ) -> Result<NodePoolRewardResult> {
        node_pool::handler_query_node_pool_reward(ctx)
    }

}
