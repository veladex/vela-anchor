use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::constants::{NFT_AUTHORITY_ADDRESS, LOCKED_VAULT_SEED};
use crate::errors::LockedVaultError;
use crate::structs::{LockedTokenVault, GlobalState, ReferralStorage};

/// Initialize global state (combines LockedTokenVault + GlobalState)
#[derive(Accounts)]
pub struct InitializeGlobal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    // ============ LockedTokenVault related accounts ============
    #[account(
        init,
        payer = authority,
        space = LockedTokenVault::SIZE,
        seeds = [LOCKED_VAULT_SEED, token_mint.key().as_ref()],
        bump
    )]
    pub locked_vault: Account<'info, LockedTokenVault>,

    #[account(
        init,
        payer = authority,
        token::mint = token_mint,
        token::authority = locked_vault,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_mint: Account<'info, Mint>,

    // ============ GlobalState related accounts ============
    #[account(
        init,
        payer = authority,
        space = GlobalState::SIZE,
        seeds = [GlobalState::SEED_PREFIX],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    // ============ System programs ============
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

/// Handler for initialize_global instruction
pub fn handler_initialize_global(ctx: Context<InitializeGlobal>) -> Result<()> {
    let clock = Clock::get()?;

    // 1. Verify authority
    require!(
        ctx.accounts.authority.key().to_string() == NFT_AUTHORITY_ADDRESS,
        LockedVaultError::UnauthorizedAuthority
    );

    // 2. Initialize LockedTokenVault
    let locked_vault = &mut ctx.accounts.locked_vault;
    locked_vault.token_mint = ctx.accounts.token_mint.key();
    locked_vault.vault_token_account = ctx.accounts.vault_token_account.key();
    locked_vault.authority = ctx.accounts.authority.key();
    locked_vault.total_locked = 0;
    locked_vault.created_at = clock.unix_timestamp;
    locked_vault.bump = ctx.bumps.locked_vault;

    // 3. Initialize GlobalState
    let global_state = &mut ctx.accounts.global_state;
    global_state.authority = ctx.accounts.authority.key();
    global_state.stake_token_mint = ctx.accounts.token_mint.key();
    global_state.total_staked = 0;
    global_state.total_interest_paid = 0;
    global_state.created_at = clock.unix_timestamp;
    global_state.bump = ctx.bumps.global_state;
    global_state.reserved = [0u8; 7];

    // Initialize production reduction related fields
    global_state.total_output = 0;
    global_state.reduction_count = 0;

    // Initialize daily deposit cap related fields
    global_state.daily_deposit_cap = crate::constants::INITIAL_DAILY_DEPOSIT_CAP;
    global_state.current_deposit_day = (clock.unix_timestamp as u64) / crate::constants::REAL_SECONDS_PER_DAY;
    global_state.daily_deposited = 0;

    // Initialize node reward pool related fields
    global_state.current_week_number = 0;
    global_state.diamond_pool_current = 0;
    global_state.gold_pool_current = 0;
    global_state.diamond_pool_previous = 0;
    global_state.gold_pool_previous = 0;
    global_state.diamond_pool_claimed_count = 0;
    global_state.gold_pool_claimed_count = 0;

    // Initialize staking statistics fields
    global_state.stats_current_day = (clock.unix_timestamp as u64) / crate::constants::REAL_SECONDS_PER_DAY;
    global_state.today_staked_amount = 0;
    global_state.last_7days_staked = [0; 7];

    // 4. Compute and write the 9 ReferralStorage PDA addresses
    // find_program_address is called only once during initialization; subsequent validation is purely key comparison
    for i in 0u8..9u8 {
        let idx = i + 1; // 1-9
        let (pda, _bump) = Pubkey::find_program_address(
            &[ReferralStorage::SEED_PREFIX, &[idx]],
            ctx.program_id,
        );
        global_state.storage_pdas[i as usize] = pda;
        msg!("storage_pdas[{}] = {}", i, pda);
    }

    msg!("========================================");
    msg!("Global initialization completed successfully");
    msg!("========================================");
    msg!("Authority: {}", global_state.authority);
    msg!("Token mint: {}", locked_vault.token_mint);
    msg!("Vault token account: {}", locked_vault.vault_token_account);
    msg!("GlobalState PDA bump: {}", global_state.bump);
    msg!("LockedVault PDA bump: {}", locked_vault.bump);
    msg!("9 storage PDA addresses written to GlobalState");
    msg!("========================================");

    Ok(())
}
