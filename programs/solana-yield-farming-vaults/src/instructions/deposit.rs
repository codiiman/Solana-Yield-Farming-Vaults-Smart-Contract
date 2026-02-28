use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::VaultError;
use crate::utils::calculate_shares_to_mint;
use crate::events::DepositEvent;

/// Deposit assets into a vault and receive shares
pub fn deposit(
    ctx: Context<Deposit>,
    amount: u64,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;
    
    // Check vault is not paused
    require!(!vault.paused, VaultError::VaultPaused);
    
    // Check minimum deposit
    require!(
        amount >= vault.min_deposit,
        VaultError::DepositTooSmall
    );
    
    // Calculate shares to mint
    let shares_to_mint = calculate_shares_to_mint(
        amount,
        vault.total_assets,
        vault.total_shares,
    )?;
    
    require!(shares_to_mint > 0, VaultError::MathOverflow);
    
    // Transfer tokens from user to vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.vault_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;
    
    // Mint vault shares to user
    let seeds = &[
        b"vault",
        &vault.vault_id.to_le_bytes(),
        &[vault.bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = MintTo {
        mint: ctx.accounts.share_mint.to_account_info(),
        to: ctx.accounts.user_share_account.to_account_info(),
        authority: ctx.accounts.vault.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::mint_to(cpi_ctx, shares_to_mint)?;
    
    // Update vault state
    vault.total_assets = vault.total_assets
        .checked_add(amount)
        .ok_or(VaultError::MathOverflow)?;
    vault.total_shares = vault.total_shares
        .checked_add(shares_to_mint)
        .ok_or(VaultError::MathOverflow)?;
    
    // Update high water mark if this is first deposit or NAV increased
    let nav_per_share = if vault.total_shares > 0 {
        vault.total_assets
            .checked_div(vault.total_shares)
            .unwrap_or(0)
    } else {
        0
    };
    
    if vault.high_water_mark == 0 || nav_per_share > vault.high_water_mark {
        vault.high_water_mark = nav_per_share;
    }
    
    emit!(DepositEvent {
        vault: vault.key(),
        user: ctx.accounts.user.key(),
        amount,
        shares_minted: shares_to_mint,
        total_assets: vault.total_assets,
        total_shares: vault.total_shares,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Deposited {} tokens, minted {} shares", amount, shares_to_mint);
    
    Ok(())
}

#[derive(Accounts)]
pub struct Deposit<'info> {
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
