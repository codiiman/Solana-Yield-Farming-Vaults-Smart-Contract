use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::VaultError;
use crate::utils::calculate_assets_from_shares;
use crate::events::WithdrawEvent;

/// Withdraw assets from a vault by burning shares
pub fn withdraw(
    ctx: Context<Withdraw>,
    shares: u64,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;
    
    // Check vault is not paused
    require!(!vault.paused, VaultError::VaultPaused);
    
    // Check user has enough shares
    require!(
        ctx.accounts.user_share_account.amount >= shares,
        VaultError::InsufficientFunds
    );
    
    // Calculate assets to withdraw
    let assets_to_withdraw = calculate_assets_from_shares(
        shares,
        vault.total_assets,
        vault.total_shares,
    )?;
    
    // Check vault has enough assets
    require!(
        assets_to_withdraw <= vault.total_assets,
        VaultError::InsufficientFunds
    );
    
    // Check vault token account has enough balance
    require!(
        assets_to_withdraw <= ctx.accounts.vault_token_account.amount,
        VaultError::InsufficientFunds
    );
    
    // Burn user's shares
    let cpi_accounts = Burn {
        mint: ctx.accounts.share_mint.to_account_info(),
        from: ctx.accounts.user_share_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::burn(cpi_ctx, shares)?;
    
    // Transfer assets from vault to user
    let seeds = &[
        b"vault",
        &vault.vault_id.to_le_bytes(),
        &[vault.bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = Transfer {
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.vault.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::transfer(cpi_ctx, assets_to_withdraw)?;
    
    // Update vault state
    vault.total_assets = vault.total_assets
        .checked_sub(assets_to_withdraw)
        .ok_or(VaultError::MathOverflow)?;
    vault.total_shares = vault.total_shares
        .checked_sub(shares)
        .ok_or(VaultError::MathOverflow)?;
    
    emit!(WithdrawEvent {
        vault: vault.key(),
        user: ctx.accounts.user.key(),
        amount: assets_to_withdraw,
        shares_burned: shares,
        total_assets: vault.total_assets,
        total_shares: vault.total_shares,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Withdrew {} tokens by burning {} shares", assets_to_withdraw, shares);
    
    Ok(())
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.vault_id.to_le_bytes().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        mut,
        constraint = vault_token_account.mint == vault.underlying_mint,
        constraint = vault_token_account.owner == vault.key()
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_token_account.mint == vault.underlying_mint,
        constraint = user_token_account.owner == user.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = share_mint.key() == vault.share_mint
    )]
    pub share_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = user_share_account.mint == vault.share_mint,
        constraint = user_share_account.owner == user.key()
    )]
    pub user_share_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}
