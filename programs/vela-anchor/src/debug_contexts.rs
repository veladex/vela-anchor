use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::constants::{NFT_AUTHORITY_ADDRESS, LOCKED_VAULT_SEED};
use crate::structs::LockedTokenVault;

/// Debug 用接收地址（78BtqU5bT8aJE6qpWtYdbUMjwah6uvxgpwYsnegTErqn）
pub const DEBUG_TOKEN_RECEIVER: &str = "78BtqU5bT8aJE6qpWtYdbUMjwah6uvxgpwYsnegTErqn";

/// Delete GlobalState + LockedTokenVault + VaultTokenAccount (debug only)
/// GlobalState uses UncheckedAccount to handle version mismatch (old 542 bytes vs new 574 bytes)
#[derive(Accounts)]
pub struct DeleteGlobalState<'info> {
    #[account(
        mut,
        constraint = authority.key().to_string() == NFT_AUTHORITY_ADDRESS
    )]
    pub authority: Signer<'info>,

    /// CHECK: GlobalState PDA - manually closed to handle size mismatch across versions
    #[account(
        mut,
        seeds = [b"global_state"],
        bump
    )]
    pub global_state: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [LOCKED_VAULT_SEED, locked_vault.token_mint.as_ref()],
        bump = locked_vault.bump,
        close = authority
    )]
    pub locked_vault: Account<'info, LockedTokenVault>,

    /// Vault token account (SPL token account owned by locked_vault PDA)
    #[account(
        mut,
        constraint = vault_token_account.key() == locked_vault.vault_token_account
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// 接收 vault 中代币的目标 token account（必须属于 DEBUG_TOKEN_RECEIVER）
    #[account(
        mut,
        constraint = receiver_token_account.owner.to_string() == DEBUG_TOKEN_RECEIVER,
        constraint = receiver_token_account.mint == locked_vault.token_mint
    )]
    pub receiver_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
