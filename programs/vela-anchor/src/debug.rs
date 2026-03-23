use anchor_lang::prelude::*;
use anchor_spl::token;
use crate::debug_contexts::*;
use crate::constants::LOCKED_VAULT_SEED;
use crate::zero_copy_storage;

/// Handler: delete GlobalState + LockedTokenVault + VaultTokenAccount
pub fn handler_delete_global_state(ctx: Context<DeleteGlobalState>) -> Result<()> {
    let token_mint_key = ctx.accounts.locked_vault.token_mint;
    let bump = ctx.accounts.locked_vault.bump;
    let signer_seeds: &[&[&[u8]]] = &[
        &[LOCKED_VAULT_SEED, token_mint_key.as_ref(), &[bump]],
    ];

    // 1. Transfer all tokens from vault to receiver (78BtqU5bT8aJE6qpWtYdbUMjwah6uvxgpwYsnegTErqn)
    let vault_balance = ctx.accounts.vault_token_account.amount;
    if vault_balance > 0 {
        let transfer_accounts = token::Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.receiver_token_account.to_account_info(),
            authority: ctx.accounts.locked_vault.to_account_info(),
        };
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );
        token::transfer(transfer_ctx, vault_balance)?;
        msg!("DEBUG: Transferred {} tokens to receiver", vault_balance);
    }

    // 2. Close vault_token_account (now balance is 0)
    let close_accounts = token::CloseAccount {
        account: ctx.accounts.vault_token_account.to_account_info(),
        destination: ctx.accounts.authority.to_account_info(),
        authority: ctx.accounts.locked_vault.to_account_info(),
    };
    let close_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        close_accounts,
        signer_seeds,
    );
    token::close_account(close_ctx)?;

    // 3. Manually close GlobalState (UncheckedAccount, transfer lamports back to authority)
    let global_state_info = ctx.accounts.global_state.to_account_info();
    let authority_info = ctx.accounts.authority.to_account_info();
    let lamports = global_state_info.lamports();
    **global_state_info.try_borrow_mut_lamports()? = 0;
    **authority_info.try_borrow_mut_lamports()? = authority_info
        .lamports()
        .checked_add(lamports)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    global_state_info.assign(&anchor_lang::system_program::ID);
    global_state_info.resize(0)?;

    msg!("========================================");
    msg!("DEBUG: GlobalState deleted (manual close)");
    msg!("DEBUG: LockedTokenVault deleted (anchor close)");
    msg!("DEBUG: VaultTokenAccount closed (CPI)");
    msg!("DEBUG: All rent returned to authority");
    msg!("========================================");

    // 4. LockedTokenVault is closed via Anchor `close = authority` constraint

    Ok(())
}

/// Handler: 强制更新 root 节点的 wallet 地址（debug only）
pub fn handler_force_update_root_wallet(
    ctx: Context<ForceUpdateRootWallet>,
    new_root_wallet: Pubkey,
) -> Result<()> {
    let storage_info = ctx.accounts.storage_1.to_account_info();
    let mut data = storage_info.try_borrow_mut_data()?;
    let mut root_record = zero_copy_storage::read_record(&data, 0)?;

    let old_wallet = root_record.wallet;
    msg!("DEBUG: Old root wallet: {}", old_wallet);
    msg!("DEBUG: New root wallet: {}", new_root_wallet);

    root_record.wallet = new_root_wallet;
    zero_copy_storage::write_record(&mut data, 0, &root_record)?;

    msg!("DEBUG: Root wallet updated successfully");
    Ok(())
}
