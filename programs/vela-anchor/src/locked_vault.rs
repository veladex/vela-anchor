use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
use crate::errors::LockedVaultError;
use crate::events::TokensLockedEvent;
use crate::contexts::LockTokens;

/// Lock tokens into the vault
pub fn handler_lock_tokens(ctx: Context<LockTokens>, amount: u64) -> Result<()> {
    // Verify amount
    require!(amount > 0, LockedVaultError::InvalidAmount);

    // Verify token mint matches
    require!(
        ctx.accounts.user_token_account.mint == ctx.accounts.locked_vault.token_mint,
        LockedVaultError::InvalidTokenMint
    );

    // Transfer tokens from user to vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.vault_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    // Update total locked amount
    let locked_vault = &mut ctx.accounts.locked_vault;
    locked_vault.total_locked = locked_vault.total_locked.checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let clock = Clock::get()?;

    // Emit event
    emit!(TokensLockedEvent {
        user: ctx.accounts.user.key(),
        amount,
        total_locked: locked_vault.total_locked,
        timestamp: clock.unix_timestamp,
    });

    msg!("Tokens locked successfully");
    msg!("User: {}", ctx.accounts.user.key());
    msg!("Amount: {}", amount);
    msg!("Total locked: {}", locked_vault.total_locked);

    Ok(())
}
