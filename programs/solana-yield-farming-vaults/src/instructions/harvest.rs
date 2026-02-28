use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::VaultError;
use crate::utils::{can_harvest, calculate_management_fee, calculate_performance_fee, estimate_apy};
use crate::events::{HarvestEvent, FeeCollectionEvent};

/// Harvest rewards and auto-compound them back into the vault
/// This can be called by anyone (permissionless) to incentivize compounding
pub fn harvest(
    ctx: Context<Harvest>,
    rewards_amount: u64, // Amount of rewards harvested (in underlying token)
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;
    
    // Check vault is not paused
    require!(!vault.paused, VaultError::VaultPaused);
    
    // Check harvest cooldown
    require!(
        can_harvest(vault, clock.unix_timestamp)?,
        VaultError::HarvestNotReady
    );
    
    if rewards_amount == 0 {
        return Err(VaultError::HarvestNotReady.into());
    }
    
    // Calculate time elapsed since last harvest
    let time_elapsed = clock.unix_timestamp
        .checked_sub(vault.last_harvest)
        .ok_or(VaultError::InvalidTimestamp)?;
    
    // Calculate and accrue management fees
    let management_fee = calculate_management_fee(
        vault.total_assets,
        vault.management_fee_bps,
        time_elapsed,
    )?;
    
    vault.accrued_management_fees = vault.accrued_management_fees
        .checked_add(management_fee)
        .ok_or(VaultError::MathOverflow)?;
    
    // Calculate performance fee on gains
    let current_nav = if vault.total_shares > 0 {
        vault.total_assets
            .checked_div(vault.total_shares)
            .unwrap_or(0)
    } else {
        0
    };
    
    let performance_fee = calculate_performance_fee(
        current_nav,
        vault.high_water_mark,
        vault.performance_fee_bps,
    )?;
    
    vault.accrued_performance_fees = vault.accrued_performance_fees
        .checked_add(performance_fee)
        .ok_or(VaultError::MathOverflow)?;
    
    // Net rewards after fees (simplified: assume fees are taken from rewards)
    let total_fees = management_fee
        .checked_add(performance_fee)
        .ok_or(VaultError::MathOverflow)?;
    
    let rewards_after_fees = if rewards_amount > total_fees {
        rewards_amount
            .checked_sub(total_fees)
            .ok_or(VaultError::MathOverflow)?
    } else {
        0
    };
    
    // Transfer rewards to vault (in production, this would come from yield source)
    // For now, assume rewards are already in the rewards_token_account
    if rewards_after_fees > 0 && ctx.accounts.rewards_token_account.amount >= rewards_after_fees {
        let cpi_accounts = Transfer {
            from: ctx.accounts.rewards_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.rewards_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, rewards_after_fees)?;
    }
    
    // Update vault state
    let assets_before = vault.total_assets;
    vault.total_assets = vault.total_assets
        .checked_add(rewards_after_fees)
        .ok_or(VaultError::MathOverflow)?;
    vault.last_harvest = clock.unix_timestamp;
    
    // Update high water mark
    let new_nav = if vault.total_shares > 0 {
        vault.total_assets
            .checked_div(vault.total_shares)
            .unwrap_or(0)
    } else {
        0
    };
    
    if new_nav > vault.high_water_mark {
        vault.high_water_mark = new_nav;
    }
    
    // Estimate APY
    let apy_estimate = estimate_apy(
        rewards_after_fees,
        assets_before,
        time_elapsed.max(1),
    ).unwrap_or(0);
    
    emit!(HarvestEvent {
        vault: vault.key(),
        harvester: ctx.accounts.harvester.key(),
        rewards_harvested: rewards_amount,
        rewards_reinvested: rewards_after_fees,
        new_total_assets: vault.total_assets,
        apy_estimate,
        timestamp: clock.unix_timestamp,
    });
    
    if total_fees > 0 {
        emit!(FeeCollectionEvent {
            vault: vault.key(),
            management_fee,
            performance_fee,
            total_fees,
            treasury: ctx.accounts.global_state.treasury,
            timestamp: clock.unix_timestamp,
        });
    }
    
    msg!("Harvested {} rewards, reinvested {} after fees", 
         rewards_amount, rewards_after_fees);
    
    Ok(())
}

/// Collect accrued fees to treasury
pub fn collect_fees(ctx: Context<CollectFees>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;
    
    // Only vault authority can collect fees
    require!(
        ctx.accounts.authority.key() == vault.authority,
        VaultError::Unauthorized
    );
    
    let total_fees = vault.accrued_management_fees
        .checked_add(vault.accrued_performance_fees)
        .ok_or(VaultError::MathOverflow)?;
    
    if total_fees == 0 {
        return Ok(()); // No fees to collect
    }
    
    // Transfer fees to treasury
    let seeds = &[
        b"vault",
        &vault.vault_id.to_le_bytes(),
        &[vault.bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = Transfer {
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.treasury_token_account.to_account_info(),
        authority: ctx.accounts.vault.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::transfer(cpi_ctx, total_fees)?;
    
    emit!(FeeCollectionEvent {
        vault: vault.key(),
        management_fee: vault.accrued_management_fees,
        performance_fee: vault.accrued_performance_fees,
        total_fees,
        treasury: ctx.accounts.global_state.treasury,
        timestamp: clock.unix_timestamp,
    });
    
    // Reset accrued fees
    vault.accrued_management_fees = 0;
    vault.accrued_performance_fees = 0;
    
    msg!("Collected {} fees to treasury", total_fees);
    
    Ok(())
}

#[derive(Accounts)]
pub struct Harvest<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.vault_id.to_le_bytes().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
    
    #[account(
        mut,
        constraint = vault_token_account.mint == vault.underlying_mint,
        constraint = vault_token_account.owner == vault.key()
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Rewards token account (source of rewards)
    #[account(mut)]
    pub rewards_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Authority that can transfer from rewards account
    pub rewards_authority: UncheckedAccount<'info>,
    
    pub harvester: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.vault_id.to_le_bytes().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
    
    #[account(
        mut,
        constraint = vault_token_account.mint == vault.underlying_mint,
        constraint = vault_token_account.owner == vault.key()
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Treasury token account
    #[account(mut)]
    pub treasury_token_account: Account<'info, TokenAccount>,
    
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}
