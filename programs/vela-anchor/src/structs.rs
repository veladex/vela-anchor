use anchor_lang::prelude::*;

// ============================================================================
// Referral related data structures
// ============================================================================

/// Referrer data structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ReferralData {
    /// User wallet address
    pub wallet: Pubkey,
    /// Parent ID (e.g. 1000002), 0 means root node
    pub parent_id: u32,
    /// Timestamp of creation
    pub created_at: i64,
    /// Total number of downline referrals (including self)
    pub total_referrals: u32,
    /// Total staked amount of downline (including self, in tokens)
    pub total_staked: u64,
    /// Self staked amount (excluding downlines, in tokens)
    pub self_staked: u64,
    /// Direct reward profit (5% from direct referrals, in tokens)
    pub direct_reward_profit: u64,
    /// Team reward profit (from level rewards, in tokens)
    pub team_reward_profit: u64,
}

impl ReferralData {
    /// Single record size: 32 + 4 + 8 + 4 + 8 + 8 + 8 + 8 = 80 bytes
    pub const SIZE: usize = 32 + 4 + 8 + 4 + 8 + 8 + 8 + 8;
}

/// Wallet information result (includes parent wallet address)
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WalletInfoResult {
    /// User wallet address
    pub wallet: Pubkey,
    /// Referral ID (e.g. 1000002)
    pub referral_id: u32,
    /// Parent ID (e.g. 1000002), 0 means root node
    pub parent_id: u32,
    /// Parent wallet address (Pubkey::default() if root node)
    pub parent_wallet: Pubkey,
    /// Timestamp of creation
    pub created_at: i64,
    /// Total number of downline referrals (including self)
    pub total_referrals: u32,
    /// Total staked amount of downline (including self, in tokens)
    pub total_staked: u64,
}

/// Referrer storage PDA account (zero-copy design)
///
/// Data layout:
/// - 8 bytes: discriminator (added automatically by Anchor)
/// - 1 byte:  index (PDA index 1-9)
/// - 4 bytes: count (current number of records)
/// - 3 bytes: reserved (aligned to 16 bytes)
/// - N * 56 bytes: ReferralData records (stored directly in account raw bytes)
#[account]
pub struct ReferralStorage {
    /// PDA index (1-9)
    pub index: u8,
    /// Currently stored count
    pub count: u32,
    /// Reserved field (for alignment)
    pub reserved: [u8; 3],
    // Note: no Vec<ReferralData>
    // Data is stored directly in the account's raw bytes, accessed via zero_copy_storage module
}

impl ReferralStorage {
    /// Header data size (zero-copy design): 8 (discriminator) + 1 (index) + 4 (count) + 3 (reserved) = 16 bytes
    pub const HEADER_SIZE: usize = 16;

    /// Minimum space for initialization
    pub const INIT_SPACE: usize = Self::HEADER_SIZE;

    /// Max capacity per PDA (based on ID encoding rule, 6-bit slot index)
    pub const MAX_CAPACITY: u32 = 110000;  // Max 110000 referrals per PDA, well within Solana 10MB account limit

    /// PDA seed prefix
    pub const SEED_PREFIX: &'static [u8] = b"referral_storage";

    /// Generate ID: PDA index * 1000000 + slot index
    pub fn generate_id(&self) -> u32 {
        (self.index as u32) * 1_000_000 + self.count
    }

    /// Decode ID to get PDA index and slot index
    pub fn decode_id(id: u32) -> (u8, u32) {
        let pda_index = (id / 1_000_000) as u8;
        let slot_index = id % 1_000_000;
        (pda_index, slot_index)
    }

    /// Check if there is still space
    pub fn has_space(&self) -> bool {
        self.count < Self::MAX_CAPACITY
    }

    // Note: add_referral and get_referral methods have been removed
    // Please use functions in the zero_copy_storage module for zero-copy operations
}

/// Management account, stores current active PDA index
#[account]
pub struct ReferralManager {
    /// Administrator
    pub authority: Pubkey,
    /// Current active PDA index (1-9)
    pub current_pda_index: u8,
    /// Whether initialized
    pub initialized: bool,
}

impl ReferralManager {
    /// Space size: 8 (discriminator) + 32 (authority) + 1 (current_pda_index) + 1 (initialized) = 42 bytes
    pub const SIZE: usize = 8 + 32 + 1 + 1;

    /// PDA seed
    pub const SEED: &'static [u8] = b"referral_manager";
}

// ============================================================================
// NFT related data structures
// ============================================================================

/// Diamond Collection state account (Diamond Node)
#[account]
pub struct DiamondCollectionState {
    pub authority: Pubkey,        // Primary address (only this can mint)
    pub collection_mint: Pubkey,  // Collection Mint address
    pub minted_count: u64,        // Number of NFTs minted
    pub max_supply: u64,          // Maximum supply (500)
    pub boost_percentage: u8,     // Interest boost (20%)
    pub bump: u8,                 // PDA bump
}

/// Gold Collection state account (Gold Node)
#[account]
pub struct GoldCollectionState {
    pub authority: Pubkey,        // Primary address (only this can mint)
    pub collection_mint: Pubkey,  // Collection Mint address
    pub minted_count: u64,        // Number of NFTs minted
    pub max_supply: u64,          // Maximum supply (10000)
    pub boost_percentage: u8,     // Interest boost (10%)
    pub bump: u8,                 // PDA bump
}

/// Diamond NFT ownership verification result
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DiamondOwnershipResult {
    pub owns_nft: bool,
    pub balance: u64,
    pub nft_mint: Pubkey,
    pub collection_mint: Pubkey,
}

/// Gold NFT ownership verification result
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GoldOwnershipResult {
    pub owns_nft: bool,
    pub balance: u64,
    pub nft_mint: Pubkey,
    pub collection_mint: Pubkey,
}

/// Node NFT mint data structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct NodeMintData {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}


// ============================================================================
// Wallet ID mapping related data structures
// ============================================================================

/// Wallet ID mapping account
#[account]
pub struct WalletIdMapping {
    /// Wallet address
    pub wallet: Pubkey,
    /// Corresponding referrer ID
    pub referral_id: u32,
}

impl WalletIdMapping {
    pub const SEED_PREFIX: &'static [u8] = b"wallet_id_mapping";
    /// Account size: 8 (discriminator) + 32 (wallet) + 4 (referral_id) = 44 bytes
    pub const SIZE: usize = 8 + 32 + 4;
}

// ============================================================================
// Locked Token Vault related data structures
// ============================================================================

/// Locked token vault account (permanently locked token pool)
#[account]
pub struct LockedTokenVault {
    /// Token mint address (fixed)
    pub token_mint: Pubkey,
    /// Vault token account address
    pub vault_token_account: Pubkey,
    /// Authority allowed to lock tokens
    pub authority: Pubkey,
    /// Total locked amount
    pub total_locked: u64,
    /// Creation timestamp
    pub created_at: i64,
    /// PDA bump seed
    pub bump: u8,
}

impl LockedTokenVault {
    /// Account size: 8 (discriminator) + 32 (token_mint) + 32 (vault_token_account) + 32 (authority) + 8 (total_locked) + 8 (created_at) + 1 (bump) = 121 bytes
    pub const SIZE: usize = 8 + 32 + 32 + 32 + 8 + 8 + 1;
}

// ============================================================================
// NFT Binding related data structures
// ============================================================================

/// NFT binding state account
#[account]
pub struct NftBindingState {
    /// NFT mint address
    pub nft_mint: Pubkey,        // 32
    /// Current owner wallet address
    pub owner: Pubkey,           // 32
    /// Node type: 1=Diamond, 2=Gold
    pub node_type: u8,           // 1
    /// Total release amount
    pub total_release: u64,      // 8
    /// Already released amount
    pub released_amount: u64,    // 8
    /// Initial binding timestamp (first bind, immutable)
    pub initial_bound_at: i64,   // 8
    /// Last binding timestamp (updated on rebind)
    pub last_bound_at: i64,      // 8
    /// PDA bump seed
    pub bump: u8,                // 1
    /// Last claimed node pool week number
    pub last_pool_claim_week: u64, // 8
}

impl NftBindingState {
    /// Account size: 8 (discriminator) + 32 + 32 + 1 + 8 + 8 + 8 + 8 + 1 + 8 = 114 bytes
    pub const SIZE: usize = 8 + 32 + 32 + 1 + 8 + 8 + 8 + 8 + 1 + 8;

    /// PDA seed prefix
    pub const SEED_PREFIX: &'static [u8] = b"nft_binding";
}

/// User state account (tracks user's current NFT binding)
#[account]
pub struct UserState {
    /// Currently bound NFT mint (Pubkey::default() if no binding)
    pub bound_nft_mint: Pubkey,  // 32
    /// PDA bump seed
    pub bump: u8,                // 1
}

impl UserState {
    /// Account size: 8 (discriminator) + 32 + 1 = 41 bytes
    pub const SIZE: usize = 8 + 32 + 1;

    /// PDA seed prefix
    pub const SEED_PREFIX: &'static [u8] = b"user_state";
}

/// NFT binding verification result
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BindingVerificationResult {
    /// Whether binding exists (PDA exists)
    pub has_binding: bool,
    /// Whether binding is active (owner matches and NFT is in wallet)
    pub is_active: bool,
    /// Binding info (None if PDA doesn't exist)
    pub binding_info: Option<BindingInfo>,
}

/// Binding information
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BindingInfo {
    /// NFT mint address
    pub nft_mint: Pubkey,
    /// Current owner
    pub owner: Pubkey,
    /// Node type
    pub node_type: u8,
    /// Total release amount
    pub total_release: u64,
    /// Released amount
    pub released_amount: u64,
    /// Initial binding timestamp
    pub initial_bound_at: i64,
    /// Last binding timestamp
    pub last_bound_at: i64,
}

/// Token release query result
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TokenReleaseInfo {
    /// Current claimable amount
    pub releasable_amount: u64,
    /// Total released amount
    pub total_released: u64,
    /// Total release limit
    pub total_release: u64,
    /// Binding days
    pub binding_days: u64,
}

/// Community status query result
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CommunityStatusResult {
    /// Referral ID
    pub referral_id: u32,
    /// Parent ID
    pub parent_id: u32,
    /// Self staked amount
    pub self_staked: u64,
    /// Total staked amount (including downlines)
    pub total_staked: u64,
    /// Direct reward profit (5% from direct referrals)
    pub direct_reward_profit: u64,
    /// Team reward profit (from level rewards)
    pub team_reward_profit: u64,
    /// Total community profit (sum of direct + team)
    pub total_community_profit: u64,
    /// Community level (L0~L7)
    pub level: u8,
}

/// Pending interest query result
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PendingInterestResult {
    /// Base interest (without NFT boost, before tax)
    pub base_interest: u64,
    /// NFT boost interest
    pub boost_interest: u64,
    /// Total interest (base + boost, before tax)
    pub total_interest: u64,
    /// Net interest received by user after tax
    pub after_tax: u64,
    /// Tax amount
    pub tax_amount: u64,
}

/// Current rates query result
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CurrentRatesResult {
    /// Total cumulative output across the network
    pub total_output: u64,
    /// Current reduction count
    pub reduction_count: u16,
    /// 7-day lock daily interest rate (RATE_BASIS_POINTS precision, 1_000_000 = 100%)
    pub rate_7d: u64,
    /// 30-day lock daily interest rate (RATE_BASIS_POINTS precision)
    pub rate_30d: u64,
    /// 90-day lock daily interest rate (RATE_BASIS_POINTS precision)
    pub rate_90d: u64,
}

// ============================================================================
// Global State related data structures (Staking)
// ============================================================================

/// Global state account (PDA) - stores global data for staking
#[account]
pub struct GlobalState {
    /// Administrator address
    pub authority: Pubkey,                      // 32 bytes
    /// Staking token mint address (must match locked_vault)
    pub stake_token_mint: Pubkey,               // 32 bytes
    /// Total staked principal
    pub total_staked: u64,                      // 8 bytes
    /// Total interest paid
    pub total_interest_paid: u64,               // 8 bytes
    /// Global state creation timestamp
    pub created_at: i64,                        // 8 bytes
    /// PDA bump seed
    pub bump: u8,                               // 1 byte
    /// Reserved field (alignment padding)
    pub reserved: [u8; 7],                      // 7 bytes (padded to 8-byte alignment)
    /// 9 ReferralStorage PDA addresses (computed at init, compared by key during validation)
    pub storage_pdas: [Pubkey; 9],              // 288 bytes
    /// Total cumulative output across the network (sum of all interest pool disbursements: interest + rewards + tax)
    /// Unit: smallest token unit (9 decimals)
    pub total_output: u64,                      // 8 bytes
    /// Current reduction count (number of halvings applied so far)
    pub reduction_count: u16,                   // 2 bytes
    /// Current daily deposit cap (dynamically adjusted, initial 3 million tokens * 10^9)
    pub daily_deposit_cap: u64,                 // 8 bytes
    /// Current statistics day (UTC day number: unix_timestamp / 86400)
    pub current_deposit_day: u64,               // 8 bytes
    /// Total deposited amount for the current day
    pub daily_deposited: u64,                   // 8 bytes
    // === Node pool fields ===
    /// Current week number
    pub current_week_number: u64,               // 8 bytes
    /// Diamond pool current week accumulation (in progress, not claimable)
    pub diamond_pool_current: u64,              // 8 bytes
    /// Gold pool current week accumulation (in progress, not claimable)
    pub gold_pool_current: u64,                 // 8 bytes
    /// Diamond pool previous week amount (claimable)
    pub diamond_pool_previous: u64,             // 8 bytes
    /// Gold pool previous week amount (claimable)
    pub gold_pool_previous: u64,                // 8 bytes
    /// Diamond pool previous week claimed count
    pub diamond_pool_claimed_count: u16,        // 2 bytes
    /// Gold pool previous week claimed count
    pub gold_pool_claimed_count: u16,           // 2 bytes
    // === Staking statistics fields ===
    /// Current statistics day for staking (UTC day number: unix_timestamp / 86400)
    pub stats_current_day: u64,                 // 8 bytes
    /// Today's total staked amount (cumulative amount from all create_stake calls today)
    pub today_staked_amount: u64,               // 8 bytes
    /// Last 7 days staked amounts (fixed index array)
    /// [0] = 7 days ago, [1] = 6 days ago, ..., [6] = today (real-time updated)
    pub last_7days_staked: [u64; 7],            // 56 bytes (7 * 8)
}

impl GlobalState {
    /// PDA seed prefix
    pub const SEED_PREFIX: &'static [u8] = b"global_state";

    /// Account size: 8 (discriminator) + 32 + 32 + 8 + 8 + 8 + 1 + 7 + 288 + 8 + 2 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 2 + 2 + 8 + 8 + 56 = 542 bytes
    pub const SIZE: usize = 542;
}

// ============================================================================
// Staking Order related data structures
// ============================================================================

/// Single staking order
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct StakeOrder {
    /// Staking amount (in smallest unit)
    pub amount: u64,                    // 8 bytes
    /// Staking period type (1=7 days, 2=30 days, 3=90 days)
    pub period_type: u8,                // 1 byte
    /// Staking start timestamp
    pub start_time: i64,                // 8 bytes
    /// Staking end timestamp
    pub end_time: i64,                  // 8 bytes
    /// Last interest calculation timestamp
    pub last_interest_time: i64,        // 8 bytes
    /// CI-08: Accumulated unclaimed base interest (without NFT boost)
    /// Zeroed on claim/unstake; new_interest is added hourly in between
    pub accumulated_interest: u64,      // 8 bytes
    /// CI-08: Cumulative total pre-tax interest claimed (including NFT boost)
    /// On each claim/unstake: += total_interest (base + boost)
    pub claimed_interest: u64,          // 8 bytes
    /// Order status (0=active, 1=completed, 2=cancelled)
    pub status: u8,                     // 1 byte
    /// Initial daily interest rate at order creation time (in RATE_BASIS_POINTS, 1_000_000 = 100%)
    /// e.g. 7-day: 5000 (0.5%), 30-day: 7000 (0.7%), 90-day: 10000 (1.0%)
    /// Rate is snapshot at stake creation; existing orders are NOT affected by subsequent reductions.
    pub initial_daily_rate: u64,        // 8 bytes
    /// Reserved field (for alignment)
    pub reserved: [u8; 5],              // 5 bytes
}

impl StakeOrder {
    /// Single order size: 8 + 1 + 8 + 8 + 8 + 8 + 8 + 1 + 8 + 5 = 63 bytes (padded to 64)
    pub const SIZE: usize = 64;
}

/// User staking account (PDA)
#[account]
pub struct UserStakeAccount {
    /// User wallet address
    pub owner: Pubkey,                          // 32 bytes
    /// Current active order count
    pub active_count: u8,                       // 1 byte
    /// Total staked principal (sum of all active orders)
    pub total_principal: u64,                   // 8 bytes
    /// Total claimed interest
    pub total_claimed_interest: u64,            // 8 bytes
    /// PDA bump seed
    pub bump: u8,                               // 1 byte
    /// Reserved field
    pub reserved: [u8; 6],                      // 6 bytes
    /// Staking orders array (fixed 15 slots to avoid stack overflow)
    pub orders: [StakeOrder; 15],               // 15 × 64 = 960 bytes
}

impl UserStakeAccount {
    /// PDA seed prefix
    pub const SEED_PREFIX: &'static [u8] = b"user_stake";

    /// Account size: 8 (discriminator) + 32 + 1 + 8 + 8 + 1 + 6 + 960 = 1024 bytes
    pub const SIZE: usize = 8 + 32 + 1 + 8 + 8 + 1 + 6 + 960;
}

// ============================================================================
// Node Pool related data structures
// ============================================================================

/// Node pool status query result
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct NodePoolStatusResult {
    /// Current week number
    pub current_week_number: u64,
    /// Current week diamond pool accumulation (not claimable)
    pub diamond_pool_current: u64,
    /// Current week gold pool accumulation (not claimable)
    pub gold_pool_current: u64,
    /// Previous week diamond pool amount (claimable)
    pub diamond_pool_previous: u64,
    /// Previous week gold pool amount (claimable)
    pub gold_pool_previous: u64,
    /// Diamond pool previous week claimed count
    pub diamond_pool_claimed_count: u16,
    /// Gold pool previous week claimed count
    pub gold_pool_claimed_count: u16,
    /// Diamond per-share claimable amount
    pub diamond_per_share: u64,
    /// Gold per-share claimable amount
    pub gold_per_share: u64,
    /// Whether the user has already claimed previous week rewards
    pub user_already_claimed: bool,
}

/// Node pool reward query result (per user)
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct NodePoolRewardResult {
    /// Week number of the reward (previous week)
    pub week_number: u64,
    /// Node type (DIAMOND or GOLD)
    pub node_type: u8,
    /// Claimable reward amount (0 if already claimed or no reward)
    pub reward_amount: u64,
    /// Whether the user has already claimed this week's rewards
    pub is_claimed: bool,
    /// Diamond per-share claimable amount
    pub diamond_per_share: u64,
    /// Gold per-share claimable amount
    pub gold_per_share: u64,
}
