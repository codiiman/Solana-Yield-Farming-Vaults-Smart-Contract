use anchor_lang::prelude::*;
use crate::errors::VaultError;
use crate::state::Vault;

/// Calculate shares to mint for a given deposit amount
/// Formula: shares = (deposit * total_shares) / total_assets
/// If vault is empty: shares = deposit (1:1 initial ratio)
pub fn calculate_shares_to_mint(
    deposit_amount: u64,
    total_assets: u64,
    total_shares: u64,
) -> Result<u64> {
    if total_shares == 0 {
        // First deposit: 1:1 ratio
        return Ok(deposit_amount);
    }
    
    if total_assets == 0 {
        return Err(VaultError::MathOverflow.into());
    }
    
    // shares = (deposit * total_shares) / total_assets
    // Use checked math to prevent overflow
    deposit_amount
        .checked_mul(total_shares)
        .ok_or(VaultError::MathOverflow)?
        .checked_div(total_assets)
        .ok_or(VaultError::MathOverflow.into())
}

/// Calculate assets to withdraw for a given number of shares
/// Formula: assets = (shares * total_assets) / total_shares
pub fn calculate_assets_from_shares(
    shares: u64,
    total_assets: u64,
    total_shares: u64,
) -> Result<u64> {
    if total_shares == 0 {
        return Err(VaultError::MathOverflow.into());
    }
    
    shares
        .checked_mul(total_assets)
        .ok_or(VaultError::MathOverflow)?
        .checked_div(total_shares)
        .ok_or(VaultError::MathOverflow.into())
}

/// Calculate management fee accrued over time
/// Formula: fee = total_assets * management_fee_bps * time_elapsed / (10000 * seconds_per_year)
pub fn calculate_management_fee(
    total_assets: u64,
    management_fee_bps: u16,
    time_elapsed_seconds: i64,
) -> Result<u64> {
    const SECONDS_PER_YEAR: i64 = 31536000; // 365 * 24 * 60 * 60
    
    if time_elapsed_seconds <= 0 {
        return Ok(0);
    }
    
    // fee = (total_assets * fee_bps * time) / (10000 * seconds_per_year)
    let fee_bps_u64 = management_fee_bps as u64;
    let time_u64 = time_elapsed_seconds as u64;
    
    total_assets
        .checked_mul(fee_bps_u64)
        .ok_or(VaultError::MathOverflow)?
        .checked_mul(time_u64)
        .ok_or(VaultError::MathOverflow)?
        .checked_div(10000 * SECONDS_PER_YEAR as u64)
        .ok_or(VaultError::MathOverflow.into())
}

/// Calculate performance fee on gains above high water mark
/// Formula: fee = (current_nav - high_water_mark) * performance_fee_bps / 10000
pub fn calculate_performance_fee(
    current_nav: u64,
    high_water_mark: u64,
    performance_fee_bps: u16,
) -> Result<u64> {
    if current_nav <= high_water_mark {
        return Ok(0);
    }
    
    let gains = current_nav
        .checked_sub(high_water_mark)
        .ok_or(VaultError::MathOverflow)?;
    
    gains
        .checked_mul(performance_fee_bps as u64)
        .ok_or(VaultError::MathOverflow)?
        .checked_div(10000)
        .ok_or(VaultError::MathOverflow.into())
}

/// Calculate health factor for leveraged positions
/// Formula: health = (collateral * collateral_factor) / debt
/// Returns basis points (e.g., 15000 = 1.5x = safe)
pub fn calculate_health_factor(
    collateral: u64,
    debt: u64,
    collateral_factor_bps: u16,
) -> Result<u64> {
    if debt == 0 {
        return Ok(u64::MAX); // No debt = infinite health
    }
    
    collateral
        .checked_mul(collateral_factor_bps as u64)
        .ok_or(VaultError::MathOverflow)?
        .checked_div(debt)
        .ok_or(VaultError::MathOverflow.into())
}

/// Check if rebalance is needed based on current vs target allocations
/// Returns true if deviation exceeds threshold
pub fn should_rebalance(
    current_allocations: &[u16; 4],
    target_allocations: &[u16; 4],
    threshold_bps: u16,
) -> bool {
    for i in 0..4 {
        let current = current_allocations[i] as i32;
        let target = target_allocations[i] as i32;
        let deviation = (current - target).abs() as u16;
        
        if deviation > threshold_bps {
            return true;
        }
    }
    false
}

/// Calculate APY estimate based on recent harvests
/// Simplified: APY = (rewards_per_period / total_assets) * periods_per_year * 10000
pub fn estimate_apy(
    rewards_harvested: u64,
    total_assets: u64,
    period_seconds: i64,
) -> Result<u64> {
    const SECONDS_PER_YEAR: i64 = 31536000;
    
    if total_assets == 0 || period_seconds <= 0 {
        return Ok(0);
    }
    
    // APY in basis points = (rewards / assets) * (seconds_per_year / period) * 10000
    let periods_per_year = SECONDS_PER_YEAR
        .checked_div(period_seconds)
        .ok_or(VaultError::MathOverflow)? as u64;
    
    rewards_harvested
        .checked_mul(10000)
        .ok_or(VaultError::MathOverflow)?
        .checked_mul(periods_per_year)
        .ok_or(VaultError::MathOverflow)?
        .checked_div(total_assets)
        .ok_or(VaultError::MathOverflow.into())
}

/// Validate oracle price freshness (stub - in production, check Pyth price age)
pub fn validate_oracle_price(
    _oracle_account: &Pubkey,
    _max_age_seconds: i64,
) -> Result<bool> {
    // TODO: In production, fetch Pyth price and check:
    // 1. Price exists and is valid
    // 2. Price timestamp is within max_age_seconds
    // 3. Price confidence interval is acceptable
    
    // For now, return true (stub)
    Ok(true)
}

/// Get price from oracle (stub - in production, read from Pyth)
pub fn get_oracle_price(_oracle_account: &Pubkey) -> Result<i64> {
    // TODO: In production, read Pyth price feed
    // Return price in scaled format (e.g., USDC price * 10^8)
    
    // Stub: return 1 USDC = 1 USDC (1e8)
    Ok(100_000_000)
}

/// Calculate leverage-adjusted position size
/// Formula: position = collateral * leverage_bps / 10000
pub fn calculate_leveraged_position(
    collateral: u64,
    leverage_bps: u16,
) -> Result<u64> {
    collateral
        .checked_mul(leverage_bps as u64)
        .ok_or(VaultError::MathOverflow)?
        .checked_div(10000)
        .ok_or(VaultError::MathOverflow.into())
}

/// Calculate debt for leveraged position
/// Formula: debt = position - collateral
pub fn calculate_debt(
    position_size: u64,
    collateral: u64,
) -> Result<u64> {
    if position_size < collateral {
        return Err(VaultError::MathOverflow.into());
    }
    
    position_size
        .checked_sub(collateral)
        .ok_or(VaultError::MathOverflow.into())
}

/// Check if vault can be harvested (cooldown expired)
pub fn can_harvest(vault: &Vault, current_timestamp: i64) -> Result<bool> {
    let time_since_harvest = current_timestamp
        .checked_sub(vault.last_harvest)
        .ok_or(VaultError::InvalidTimestamp)?;
    
    Ok(time_since_harvest >= vault.harvest_cooldown)
}

/// Check if vault can be rebalanced (cooldown expired)
pub fn can_rebalance(vault: &Vault, current_timestamp: i64) -> Result<bool> {
    let time_since_rebalance = current_timestamp
        .checked_sub(vault.last_rebalance)
        .ok_or(VaultError::InvalidTimestamp)?;
    
    Ok(time_since_rebalance >= vault.rebalance_cooldown)
}

/// Calculate NAV (Net Asset Value) per share
/// Formula: nav_per_share = total_assets / total_shares
pub fn calculate_nav_per_share(
    total_assets: u64,
    total_shares: u64,
) -> Result<u64> {
    if total_shares == 0 {
        return Ok(0);
    }
    
    total_assets
        .checked_div(total_shares)
        .ok_or(VaultError::MathOverflow.into())
}
