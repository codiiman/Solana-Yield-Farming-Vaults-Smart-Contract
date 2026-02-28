use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::VaultError;
use crate::utils::{can_rebalance, should_rebalance, validate_oracle_price, get_oracle_price};
use crate::events::RebalanceEvent;

/// Rebalance vault positions to match target allocations
/// This adjusts the vault's asset allocation based on market conditions
pub fn rebalance(
    ctx: Context<Rebalance>,
    target_allocations: [u16; 4],
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;
    
    // Check vault is not paused
    require!(!vault.paused, VaultError::VaultPaused);
    
    // Check rebalance cooldown
    require!(
        can_rebalance(vault, clock.unix_timestamp)?,
        VaultError::RebalanceThresholdNotMet
    );
    
    // Validate target allocations sum to 10000 (100%)
    let sum: u32 = target_allocations.iter().map(|&x| x as u32).sum();
    require!(
        sum == 10000,
        VaultError::InvalidRebalanceParams
    );
    
    // Check if rebalance is needed
    let needs_rebalance = should_rebalance(
        &vault.strategy_config.current_allocations,
        &target_allocations,
        vault.rebalance_threshold_bps,
    );
    
    require!(needs_rebalance, VaultError::RebalanceThresholdNotMet);
    
    // Validate oracle if needed (for price-based rebalancing)
    if let Some(oracle) = vault.strategy_config.oracle_price_feed {
        let is_valid = validate_oracle_price(&oracle, 300)?; // 5 min max age
        require!(is_valid, VaultError::StaleOraclePrice);
        
        // Get current price for event
        let _current_price = get_oracle_price(&oracle)?;
    }
    
    // Store assets before rebalance
    let assets_before = vault.total_assets;
    
    // In production, this would:
    // 1. Calculate current position values
    // 2. Calculate target position values based on allocations
    // 3. Execute swaps/transfers to rebalance
    // 4. Update current_allocations
    
    // For now, we'll just update the allocations (stub)
    // In production, you'd integrate with DEXs, lending protocols, etc.
    vault.strategy_config.current_allocations = target_allocations;
    vault.strategy_config.target_allocations = target_allocations;
    
    vault.last_rebalance = clock.unix_timestamp;
    
    // Get price for event (stub)
    let price_before = get_oracle_price(
        &vault.strategy_config.oracle_price_feed.unwrap_or_default()
    ).unwrap_or(100_000_000);
    let price_after = price_before; // In production, would be actual price after rebalance
    
    emit!(RebalanceEvent {
        vault: vault.key(),
        rebalancer: ctx.accounts.rebalancer.key(),
        assets_before,
        assets_after: vault.total_assets,
        rebalance_type: 0, // 0 = full rebalance
        price_before,
        price_after,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Rebalanced vault {} to new allocations", vault.vault_id);
    
    Ok(())
}

/// Update vault strategy configuration
pub fn update_strategy_config(
    ctx: Context<UpdateStrategyConfig>,
    strategy_config: StrategyConfig,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    
    // Only vault authority can update strategy
    require!(
        ctx.accounts.authority.key() == vault.authority,
        VaultError::Unauthorized
    );
    
    vault.strategy_config = strategy_config;
    
    msg!("Updated strategy config for vault {}", vault.vault_id);
    
    Ok(())
}

/// Update vault parameters (fees, cooldowns, etc.)
pub fn update_vault_params(
    ctx: Context<UpdateVaultParams>,
    management_fee_bps: Option<u16>,
    performance_fee_bps: Option<u16>,
    harvest_cooldown: Option<i64>,
    rebalance_cooldown: Option<i64>,
    rebalance_threshold_bps: Option<u16>,
    min_deposit: Option<u64>,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    
    // Only vault authority can update params
    require!(
        ctx.accounts.authority.key() == vault.authority,
        VaultError::Unauthorized
    );
    
    if let Some(fee) = management_fee_bps {
        require!(fee <= 1000, VaultError::InvalidFeeConfig);
        vault.management_fee_bps = fee;
    }
    
    if let Some(fee) = performance_fee_bps {
        require!(fee <= 5000, VaultError::InvalidFeeConfig);
        vault.performance_fee_bps = fee;
    }
    
    if let Some(cooldown) = harvest_cooldown {
        require!(cooldown >= 0, VaultError::InvalidTimestamp);
        vault.harvest_cooldown = cooldown;
    }
    
    if let Some(cooldown) = rebalance_cooldown {
        require!(cooldown >= 0, VaultError::InvalidTimestamp);
        vault.rebalance_cooldown = cooldown;
    }
    
    if let Some(threshold) = rebalance_threshold_bps {
        vault.rebalance_threshold_bps = threshold;
    }
    
    if let Some(min) = min_deposit {
        vault.min_deposit = min;
    }
    
    msg!("Updated vault parameters");
    
    Ok(())
}

#[derive(Accounts)]
pub struct Rebalance<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.vault_id.to_le_bytes().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    
    pub rebalancer: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateStrategyConfig<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.vault_id.to_le_bytes().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateVaultParams<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.vault_id.to_le_bytes().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    
    pub authority: Signer<'info>,
}
