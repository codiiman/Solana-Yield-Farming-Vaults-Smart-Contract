use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::VaultError;

/// Pause a vault (prevents deposits/withdrawals)
pub fn pause_vault(ctx: Context<PauseVault>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    
    // Only vault authority can pause
    require!(
        ctx.accounts.authority.key() == vault.authority,
        VaultError::Unauthorized
    );
    
    require!(!vault.paused, VaultError::VaultNotPaused);
    
    vault.paused = true;
    
    msg!("Vault {} paused", vault.vault_id);
    
    Ok(())
}

/// Unpause a vault
pub fn unpause_vault(ctx: Context<UnpauseVault>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    
    // Only vault authority can unpause
    require!(
        ctx.accounts.authority.key() == vault.authority,
        VaultError::Unauthorized
    );
    
    require!(vault.paused, VaultError::VaultPaused);
    
    vault.paused = false;
    
    msg!("Vault {} unpaused", vault.vault_id);
    
    Ok(())
}

#[derive(Accounts)]
pub struct PauseVault<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.vault_id.to_le_bytes().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UnpauseVault<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.vault_id.to_le_bytes().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    
    pub authority: Signer<'info>,
}
