use anchor_lang::prelude::Pubkey;
use std::str::FromStr;

// ============================================================================
// NFT related constants
// ============================================================================

/// NFT authority address (hardcoded)
pub const NFT_AUTHORITY_ADDRESS: &str = "F1w9GEWbFbUSPZqxi5yd7TVWzWvDcWQKvHyHLaSyS4XS";

/// Parse NFT_AUTHORITY_ADDRESS to Pubkey
pub fn nft_authority_pubkey() -> Pubkey {
    Pubkey::from_str(NFT_AUTHORITY_ADDRESS).unwrap()
}

/// Diamond Node NFT max supply
pub const DIAMOND_NFT_MAX_SUPPLY: u64 = 600; // 600 pieces

/// Gold Node NFT max supply
pub const GOLD_NFT_MAX_SUPPLY: u64 = 12000; // 12000 pieces

/// Diamond Node boost percentage
pub const DIAMOND_BOOST_PERCENTAGE: u8 = 20;

/// Gold Node boost percentage
pub const GOLD_BOOST_PERCENTAGE: u8 = 10;

// ============================================================================
// Referral related constants
// ============================================================================

pub const REFERRAL_MANAGER_SEED: &[u8] = b"referral_manager";
pub const REFERRAL_STORAGE_SEED: &[u8] = b"referral_storage";


// ============================================================================
// Staking related constants
// ============================================================================

/// Seconds per day
/// Production env: 86400 (24 * 60 * 60)
/// Test env: can be changed to 30 for easy debugging
pub const SECONDS_PER_DAY: u32 = 10;  // Can be changed to 30 for debugging

/// Seconds per hour
/// Production env: 3600 (60 * 60)
/// Test env: can be changed to 5 for easy debugging
pub const SECONDS_PER_HOUR: u32 = 5;  // Can be changed to 5 for debugging

/// Max simultaneous staking count per user (reduced to 15 to avoid stack overflow)
pub const MAX_STAKES_PER_USER: usize = 15;

/// Minimum stake amount (1,000 tokens, assuming 9 decimal places)
pub const MIN_STAKE_AMOUNT: u64 = 1_000_000_000_000;

/// Maximum stake amount (50,000 tokens, assuming 9 decimal places)
pub const MAX_STAKE_AMOUNT: u64 = 50_000_000_000_000;

/// Token decimals (10^9 for amount validation)
pub const AMOUNT_DECIMALS: u64 = 1_000_000_000;

/// Staking period enum values
pub const STAKE_PERIOD_7_DAYS: u8 = 1;
pub const STAKE_PERIOD_30_DAYS: u8 = 2;
pub const STAKE_PERIOD_90_DAYS: u8 = 3;

/// Staking period in seconds - using u32 to save space
pub const PERIOD_7_DAYS: u32 = 7 * SECONDS_PER_DAY;   // 7 days in seconds
pub const PERIOD_30_DAYS: u32 = 30 * SECONDS_PER_DAY; // 30 days in seconds
pub const PERIOD_90_DAYS: u32 = 90 * SECONDS_PER_DAY; // 90 days in seconds

/// Dedicated basis point denominator for daily interest rate: 1_000_000 = 100%
/// Separated from BASIS_POINTS(10000), used only for daily rate calculations
/// Benefit: hourly precision improves from 50/24=2 to 5000/24=208
pub const RATE_BASIS_POINTS: u64 = 1_000_000;

/// Daily interest rate (in RATE_BASIS_POINTS, 1_000_000 = 100%)
/// 0.5% = 5000 / 1_000_000
pub const DAILY_RATE_7_DAYS: u64 = 5_000;    // 0.5%
pub const DAILY_RATE_30_DAYS: u64 = 7_000;   // 0.7%
pub const DAILY_RATE_90_DAYS: u64 = 10_000;  // 1.0%

/// Basis point denominator (used for NFT boost / tax rate / community rewards / reduction ratios)
pub const BASIS_POINTS: u64 = 10_000;

// ============ Reduction mechanism constants ============

/// Output threshold per reduction: 20 million tokens (9 decimals)
pub const REDUCTION_THRESHOLD: u64 = 20_000_000_000_000_000; // 20 million * 10^9

/// Stage 1 end threshold: 500 million tokens
pub const STAGE_1_END: u64 = 500_000_000_000_000_000;   // 500 million * 10^9

/// Stage 2 end threshold: 800 million tokens
pub const STAGE_2_END: u64 = 800_000_000_000_000_000;   // 800 million * 10^9

/// Stage 3 end threshold (max output cap): 1 billion tokens
pub const STAGE_3_END: u64 = 1_000_000_000_000_000_000; // 1 billion * 10^9

/// Stage 1 reduction ratio: 5% (multiply by 9500 / 10000)
pub const STAGE_1_RETAIN_BPS: u64 = 9500; // Retain 95%

/// Stage 2 reduction ratio: 3% (multiply by 9700 / 10000)
pub const STAGE_2_RETAIN_BPS: u64 = 9700; // Retain 97%

/// Stage 3 reduction ratio: 2% (multiply by 9800 / 10000)
pub const STAGE_3_RETAIN_BPS: u64 = 9800; // Retain 98%

/// Stage 1 reduction count: 500 million / 20 million = 25 times
pub const STAGE_1_REDUCTIONS: u16 = 25;

/// Stage 2 reduction count: 300 million / 20 million = 15 times
pub const STAGE_2_REDUCTIONS: u16 = 15;

/// Stage 3 reduction count: 200 million / 20 million = 10 times
pub const STAGE_3_REDUCTIONS: u16 = 10;

/// Max reduction count: 25 + 15 + 10 = 50 times
pub const MAX_REDUCTIONS: u16 = 50;

/// NFT node interest boost rates (in basis points)
pub const DIAMOND_NODE_BOOST: u64 = 2000;  // 20%
pub const GOLD_NODE_BOOST: u64 = 1000;     // 10%

/// Interest tax rate (in basis points)
pub const INTEREST_TAX_RATE: u64 = 1000;   // 10%

/// Dead address (interest tax collection address)
/// Uses a common unowned address to avoid using the System Program address
pub const DEAD_ADDRESS: &str = "1nc1nerator11111111111111111111111111111111";

/// Returns the dead address as a Pubkey for constraint validation
pub fn dead_address_pubkey() -> Pubkey {
    Pubkey::from_str(DEAD_ADDRESS).unwrap()
}

/// Referral update level
pub const UPDATE_LEVEL_NUM: u32 = 60;

/// Referral relationship update levels for staking
pub const REFERRAL_UPDATE_LEVELS: u32 = 50;

/// Staking order status constants
pub const ORDER_STATUS_ACTIVE: u8 = 0;
pub const ORDER_STATUS_COMPLETED: u8 = 1;
pub const ORDER_STATUS_CANCELLED: u8 = 2;

// ============================================================================
// Locked Token Vault related constants
// ============================================================================

/// Locked token vault PDA seed
pub const LOCKED_VAULT_SEED: &[u8] = b"locked_token_vault_seed";

// ============================================================================
// NFT Binding related constants
// ============================================================================

/// NFT binding PDA seed
pub const NFT_BINDING_SEED: &[u8] = b"nft_binding";

/// User state PDA seed
pub const USER_STATE_SEED: &[u8] = b"user_state";

/// Diamond node total release amount (30,000 tokens × 10^9 decimals)
pub const DIAMOND_TOTAL_RELEASE: u64 = 30_000_000_000_000;

/// Gold node total release amount (2,250 tokens × 10^9 decimals)
pub const GOLD_TOTAL_RELEASE: u64 = 2_250_000_000_000;

/// Node type: Diamond
pub const NODE_TYPE_DIAMOND: u8 = 1;

/// Node type: Gold
pub const NODE_TYPE_GOLD: u8 = 2;

/// Binding cooldown period (15 days)
pub const BINDING_COOLDOWN_DAYS: u32 = 15;

/// Binding cooldown period in seconds
pub const BINDING_COOLDOWN_SECONDS: i64 = 15 * SECONDS_PER_DAY as i64;

/// Diamond node daily release amount (300 tokens × 10^9 decimals)
pub const DIAMOND_DAILY_RELEASE: u64 = 300_000_000_000;

/// Gold node daily release amount (22.5 tokens × 10^9 decimals)
pub const GOLD_DAILY_RELEASE: u64 = 22_500_000_000;

// ============================================================================
// Community Reward related constants
// ============================================================================

/// Direct referral reward ratio (bps): reward * 5%
pub const COMMUNITY_DIRECT_REWARD_BPS: u64 = 500;

/// Level reward pool ratio (bps): reward * 80%
pub const COMMUNITY_LEVEL_POOL_BPS: u64 = 8000;

/// Same-level bonus ratio (bps): reward * 2% (fixed, independent of level)
pub const COMMUNITY_LEVEL_BONUS_BPS: u64 = 200;

/// Max traversal depth for community rewards
pub const COMMUNITY_MAX_TRAVERSE_DEPTH: u32 = 50;

/// Number of levels (L0~L7)
pub const COMMUNITY_LEVEL_COUNT: usize = 8;

/// Minimum level for same-level bonus (starts from L3)
pub const COMMUNITY_LEVEL_BONUS_MIN: u8 = 3;

/// Self-staked requirement per level, unit: smallest token unit
/// L0=0, L1=0, L2=0, L3=100k, L4=100k, L5=200k, L6=200k, L7=300k
pub const LEVEL_SELF_STAKED_REQ: [u64; 8] = [
    0,
    0,
    0,
    100_000_000_000_000,   // 100k * 10^9
    100_000_000_000_000,   // 100k
    200_000_000_000_000,   // 200k
    200_000_000_000_000,   // 200k
    300_000_000_000_000,   // 300k
];

/// Community performance requirement per level (total_staked, subordinates only), unit: smallest token unit
/// L0=0, L1=200k, L2=600k, L3=3M, L4=6M, L5=16M, L6=40M, L7=100M
pub const LEVEL_TOTAL_STAKED_REQ: [u64; 8] = [
    0,
    200_000_000_000_000,       // 200k
    600_000_000_000_000,       // 600k
    3_000_000_000_000_000,     // 3M
    6_000_000_000_000_000,     // 6M
    16_000_000_000_000_000,    // 16M
    40_000_000_000_000_000,    // 40M
    100_000_000_000_000_000,   // 100M
];

/// Level differential share ratios (bps)
/// L0=0%, L1=10%, L2=20%, L3=30%, L4=40%, L5=50%, L6=60%, L7=70%
pub const LEVEL_DIFF_BPS: [u64; 8] = [0, 1000, 2000, 3000, 4000, 5000, 6000, 7000];

/// Root node referral_id (fixed: storage_1 slot_0)
pub const ROOT_REFERRAL_ID: u32 = 1_000_000;

// ============ Daily deposit cap constants ============

/// Real seconds per day (used for daily quota calculation, independent from interest rate SECONDS_PER_DAY)
pub const REAL_SECONDS_PER_DAY: u64 = 86400;

/// Initial daily deposit cap: 3 million tokens (9 decimals)
pub const INITIAL_DAILY_DEPOSIT_CAP: u64 = 3_000_000_000_000_000; // 3M * 10^9

/// Next-day growth ratio when daily cap is reached (110%, in BASIS_POINTS)
pub const DAILY_CAP_GROWTH_BPS: u64 = 11000; // 110% = 11000 / 10000

/// Cap exhaustion threshold: considered full when less than 1000 tokens remain
pub const DAILY_CAP_EXHAUST_THRESHOLD: u64 = 1_000_000_000_000; // 1000 * 10^9

/// Per-address base staking cap: 50k tokens
pub const USER_STAKE_CAP_BASE: u64 = 50_000_000_000_000; // 50k * 10^9

/// Total staked threshold tier 1: 45 million tokens -> per-address cap increases to 100k
pub const TOTAL_STAKED_TIER1: u64 = 45_000_000_000_000_000; // 45M * 10^9

/// Per-address staking cap (tier 1): 100k tokens
pub const USER_STAKE_CAP_TIER1: u64 = 100_000_000_000_000; // 100k * 10^9

/// Total staked threshold tier 2: 90 million tokens -> per-address cap increases to 150k
pub const TOTAL_STAKED_TIER2: u64 = 90_000_000_000_000_000; // 90M * 10^9

/// Per-address staking cap (tier 2): 150k tokens
pub const USER_STAKE_CAP_TIER2: u64 = 150_000_000_000_000; // 150k * 10^9

// ============================================================================
// Node Pool related constants
// ============================================================================

/// Diamond node pool ratio (based on BASIS_POINTS = 10000): 7.5%
pub const NODE_POOL_DIAMOND_BPS: u64 = 750;

/// Gold node pool ratio (based on BASIS_POINTS = 10000): 7.5%
pub const NODE_POOL_GOLD_BPS: u64 = 750;

/// Diamond node claim shares: 1/600
pub const DIAMOND_POOL_SHARES: u64 = 600;

/// Gold node claim shares: 1/12000
pub const GOLD_POOL_SHARES: u64 = 12_000;

/// Seconds per week (real time, not shortened)
pub const SECONDS_PER_WEEK: i64 = 7 * 86400; // 604800

/// Week epoch offset (1970-01-05 Monday 00:00 UTC)
pub const WEEK_EPOCH_OFFSET: i64 = 345600;

