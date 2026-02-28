use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Insufficient funds in vault")]
    InsufficientFunds,
    
    #[msg("Vault is paused")]
    VaultPaused,
    
    #[msg("Vault is not paused")]
    VaultNotPaused,
    
    #[msg("Invalid vault strategy")]
    InvalidStrategy,
    
    #[msg("Rebalance threshold not met")]
    RebalanceThresholdNotMet,
    
    #[msg("Rebalance failed")]
    RebalanceFailed,
    
    #[msg("Liquidation needed - health factor too low")]
    LiquidationNeeded,
    
    #[msg("Liquidation not needed - health factor safe")]
    LiquidationNotNeeded,
    
    #[msg("Invalid leverage ratio")]
    InvalidLeverage,
    
    #[msg("Maximum leverage exceeded")]
    MaxLeverageExceeded,
    
    #[msg("Oracle price stale or invalid")]
    StaleOraclePrice,
    
    #[msg("Invalid oracle account")]
    InvalidOracle,
    
    #[msg("Unauthorized - not vault authority")]
    Unauthorized,
    
    #[msg("Invalid deposit amount - below minimum")]
    DepositTooSmall,
    
    #[msg("Invalid withdrawal amount - exceeds balance")]
    WithdrawalTooLarge,
    
    #[msg("Math overflow")]
    MathOverflow,
    
    #[msg("Invalid token mint")]
    InvalidMint,
    
    #[msg("Harvest not available yet")]
    HarvestNotReady,
    
    #[msg("Invalid fee configuration")]
    InvalidFeeConfig,
    
    #[msg("Position not found")]
    PositionNotFound,
    
    #[msg("Invalid rebalance parameters")]
    InvalidRebalanceParams,
    
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    
    #[msg("Strategy not initialized")]
    StrategyNotInitialized,
    
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    
    #[msg("Compounding cooldown not expired")]
    CompoundingCooldown,
}
