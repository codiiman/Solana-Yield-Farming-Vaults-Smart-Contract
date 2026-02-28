use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::VaultError;
use crate::utils::{calculate_health_factor, validate_oracle_price, get_oracle_price};
use crate::events::{LiquidationEvent, LeverageAdjustmentEvent};

/// Liquidate an undercollateralized leveraged position
/// This is called when health factor drops below liquidation threshold
pub fn liquidate(
    ctx: Context<Liquidate>,
    collateral_to_seize: u64,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let user_position = &ctx.accounts.user_position;
    let clock = Clock::get()?;
    
    // Check vault is not paused
    require!(!vault.paused, VaultError::VaultPaused);
    
    // Validate oracle
    if let Some(oracle) = vault.strategy_config.oracle_price_feed {
        let is_valid = validate_oracle_price(&oracle, 300)?;
        require!(is_valid, VaultError::StaleOraclePrice);
    }
    
    // Calculate health factor
    let collateral_factor = vault.strategy_config
        .collateral_factor_bps
        .unwrap_or(8000); // 80% default
    
    let health_factor = calculate_health_factor(
        user_position.collateral,
        user_position.debt,
        collateral_factor,
    )?;
    
    let liquidation_threshold = vault.strategy_config
        .liquidation_threshold_bps
        .unwrap_or(11000); // 1.1x = 11000 bps
    
    require!(
        health_factor < liquidation_threshold,
        VaultError::LiquidationNotNeeded
    );
    
    // Validate collateral to seize
    require!(
        collateral_to_seize <= user_position.collateral,
        VaultError::InsufficientFunds
    );
    
    // Calculate debt to repay (with liquidation bonus)
    // Liquidation bonus: 5% (10500 bps)
    const LIQUIDATION_BONUS_BPS: u16 = 10500;
    let debt_to_repay = collateral_to_seize
        .checked_mul(LIQUIDATION_BONUS_BPS as u64)
        .ok_or(VaultError::MathOverflow)?
        .checked_div(10000)
        .ok_or(VaultError::MathOverflow)?;
    
    require!(
        debt_to_repay <= user_position.debt,
        VaultError::MathOverflow
    );
    
    // Transfer collateral from vault to liquidator
    let seeds = &[
        b"vault",
        &vault.vault_id.to_le_bytes(),
        &[vault.bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = Transfer {
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.liquidator_token_account.to_account_info(),
        authority: ctx.accounts.vault.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::transfer(cpi_ctx, collateral_to_seize)?;
    
    // In production, liquidator would repay debt to lending protocol
    // Note: UserPosition would need to be updated separately in production
    
    // Update vault state
    vault.total_assets = vault.total_assets
        .checked_sub(collateral_to_seize)
        .ok_or(VaultError::MathOverflow)?;
    
    // Update leverage
    let new_collateral = user_position.collateral
        .checked_sub(collateral_to_seize)
        .ok_or(VaultError::MathOverflow)?;
    
    let new_debt = user_position.debt
        .checked_sub(debt_to_repay)
        .ok_or(VaultError::MathOverflow)?;
    
    let new_leverage = if new_collateral > 0 {
        let position = new_collateral
            .checked_add(new_debt)
            .ok_or(VaultError::MathOverflow)?;
        position
            .checked_mul(10000)
            .ok_or(VaultError::MathOverflow)?
            .checked_div(new_collateral)
            .unwrap_or(10000)
    } else {
        10000 // 1x
    };
    
    vault.current_leverage_bps = new_leverage.min(vault.max_leverage_bps) as u16;
    
    emit!(LiquidationEvent {
        vault: vault.key(),
        liquidator: ctx.accounts.liquidator.key(),
        liquidated_user: user_position.user,
        collateral_seized: collateral_to_seize,
        debt_repaid: debt_to_repay,
        health_factor_before: health_factor,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Liquidated {} collateral, repaid {} debt", 
         collateral_to_seize, debt_to_repay);
    
    Ok(())
}

/// Adjust leverage for a leveraged vault position
pub fn adjust_leverage(
    ctx: Context<AdjustLeverage>,
    target_leverage_bps: u16,
    collateral_add: u64,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let user_position = &mut ctx.accounts.user_position;
    let clock = Clock::get()?;
    
    // Check vault is not paused
    require!(!vault.paused, VaultError::VaultPaused);
    
    // Validate leverage
    require!(
        target_leverage_bps >= 10000 && target_leverage_bps <= vault.max_leverage_bps,
        VaultError::InvalidLeverage
    );
    
    let leverage_before = user_position.leverage_bps;
    let debt_before = user_position.debt;
    
    // If adding collateral, transfer from user
    if collateral_add > 0 {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, collateral_add)?;
        
        user_position.collateral = user_position.collateral
            .checked_add(collateral_add)
            .ok_or(VaultError::MathOverflow)?;
        
        vault.total_assets = vault.total_assets
            .checked_add(collateral_add)
            .ok_or(VaultError::MathOverflow)?;
    }
    
    // Calculate new position size
    let new_position_size = user_position.collateral
        .checked_mul(target_leverage_bps as u64)
        .ok_or(VaultError::MathOverflow)?
        .checked_div(10000)
        .ok_or(VaultError::MathOverflow)?;
    
    // Calculate new debt
    let new_debt = new_position_size
        .checked_sub(user_position.collateral)
        .ok_or(VaultError::MathOverflow)?;
    
    // In production, would interact with lending protocol to adjust debt
    // For now, just update state
    user_position.debt = new_debt;
    user_position.leverage_bps = target_leverage_bps;
    user_position.last_interaction = clock.unix_timestamp;
    
    vault.current_leverage_bps = target_leverage_bps;
    
    emit!(LeverageAdjustmentEvent {
        vault: vault.key(),
        user: user_position.user,
        leverage_before,
        leverage_after: target_leverage_bps,
        collateral_added: collateral_add,
        debt_added: new_debt
            .checked_sub(debt_before)
            .unwrap_or(0),
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Adjusted leverage to {} bps", target_leverage_bps);
    
    Ok(())
}

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.vault_id.to_le_bytes().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        constraint = user_position.vault == vault.key()
    )]
    pub user_position: Account<'info, UserPosition>,
    
    #[account(
        mut,
        constraint = vault_token_account.mint == vault.underlying_mint,
        constraint = vault_token_account.owner == vault.key()
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = liquidator_token_account.mint == vault.underlying_mint,
        constraint = liquidator_token_account.owner == liquidator.key()
    )]
    pub liquidator_token_account: Account<'info, TokenAccount>,
    
    pub liquidator: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AdjustLeverage<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.vault_id.to_le_bytes().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        mut,
        constraint = user_position.user == user.key(),
        constraint = user_position.vault == vault.key()
    )]
    pub user_position: Account<'info, UserPosition>,
    
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
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}
