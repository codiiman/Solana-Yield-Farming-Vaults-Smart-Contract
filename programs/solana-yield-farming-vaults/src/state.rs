use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

/// Global protocol state - stores protocol-wide configuration
#[account]
#[derive(Default)]
pub struct GlobalState {
    /// Protocol authority (can update fees, pause, etc.)
    pub authority: Pubkey,
    
    /// Treasury account for fee collection
    pub treasury: Pubkey,
    
    /// Default management fee (basis points, e.g., 200 = 2%)
    pub default_management_fee_bps: u16,
    
    /// Default performance fee (basis points, e.g., 2000 = 20%)
    pub default_performance_fee_bps: u16,
    
    /// Protocol paused flag
    pub paused: bool,
    
    /// Total number of vaults created
    pub vault_count: u64,
    
    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl GlobalState {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        32 + // treasury
        2 +  // default_management_fee_bps
        2 +  // default_performance_fee_bps
        1 +  // paused
        8 +  // vault_count
        1;   // bump
}

/// Vault account - represents a single yield farming vault
#[account]
pub struct Vault {
    /// Vault identifier (unique per vault)
    pub vault_id: u64,
    
    /// Strategy type (0 = LP Farming, 1 = Leveraged Yield, 2 = Auto-Compound, 3 = Delta-Neutral)
    pub strategy: u8,
    
    /// Underlying asset mint (e.g., SOL, USDC)
    pub underlying_mint: Pubkey,
    
    /// Vault share token mint (represents ownership)
    pub share_mint: Pubkey,
    
    /// Vault's token account holding underlying assets
    pub vault_token_account: Pubkey,
    
    /// Vault authority (can pause, update strategy params)
    pub authority: Pubkey,
    
    /// Total assets under management (in underlying token units)
    pub total_assets: u64,
    
    /// Total shares minted
    pub total_shares: u64,
    
    /// Last harvest timestamp
    pub last_harvest: i64,
    
    /// Last rebalance timestamp
    pub last_rebalance: i64,
    
    /// Management fee (basis points)
    pub management_fee_bps: u16,
    
    /// Performance fee (basis points)
    pub performance_fee_bps: u16,
    
    /// High water mark for performance fee calculation
    pub high_water_mark: u64,
    
    /// Accumulated management fees (to be collected)
    pub accrued_management_fees: u64,
    
    /// Accumulated performance fees (to be collected)
    pub accrued_performance_fees: u64,
    
    /// Vault paused flag
    pub paused: bool,
    
    /// Minimum deposit amount
    pub min_deposit: u64,
    
    /// Maximum leverage (for leveraged strategies, in basis points: 20000 = 2x)
    pub max_leverage_bps: u16,
    
    /// Current leverage (basis points)
    pub current_leverage_bps: u16,
    
    /// Rebalance threshold (basis points: deviation from target before rebalancing)
    pub rebalance_threshold_bps: u16,
    
    /// Harvest cooldown period (seconds)
    pub harvest_cooldown: i64,
    
    /// Rebalance cooldown period (seconds)
    pub rebalance_cooldown: i64,
    
    /// Strategy-specific configuration (strategy-dependent)
    pub strategy_config: StrategyConfig,
    
    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl Vault {
    pub const LEN: usize = 8 + // discriminator
        8 +  // vault_id
        1 +  // strategy
        32 + // underlying_mint
        32 + // share_mint
        32 + // vault_token_account
        32 + // authority
        8 +  // total_assets
        8 +  // total_shares
        8 +  // last_harvest
        8 +  // last_rebalance
        2 +  // management_fee_bps
        2 +  // performance_fee_bps
        8 +  // high_water_mark
        8 +  // accrued_management_fees
        8 +  // accrued_performance_fees
        1 +  // paused
        8 +  // min_deposit
        2 +  // max_leverage_bps
        2 +  // current_leverage_bps
        2 +  // rebalance_threshold_bps
        8 +  // harvest_cooldown
        8 +  // rebalance_cooldown
        StrategyConfig::LEN + // strategy_config
        1;   // bump
}

/// Strategy-specific configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct StrategyConfig {
    /// For LP strategies: pool address
    pub pool_address: Option<Pubkey>,
    
    /// For leveraged strategies: lending protocol address
    pub lending_protocol: Option<Pubkey>,
    
    /// For leveraged strategies: collateral factor (basis points)
    pub collateral_factor_bps: Option<u16>,
    
    /// For delta-neutral: hedge position address (e.g., perp market)
    pub hedge_position: Option<Pubkey>,
    
    /// Target allocation percentages (for multi-asset strategies)
    pub target_allocations: [u16; 4], // Up to 4 assets
    
    /// Current allocations (basis points, sum to 10000)
    pub current_allocations: [u16; 4],
    
    /// Oracle price feed (Pyth)
    pub oracle_price_feed: Option<Pubkey>,
    
    /// Health factor threshold for liquidation (basis points, e.g., 11000 = 1.1x)
    pub liquidation_threshold_bps: Option<u16>,
    
    /// Reserve space for future strategy params
    pub reserved: [u8; 64],
}

impl StrategyConfig {
    pub const LEN: usize = 1 + 32 + // pool_address (Option<Pubkey>)
        1 + 32 + // lending_protocol (Option<Pubkey>)
        1 + 2 +  // collateral_factor_bps (Option<u16>)
        1 + 32 + // hedge_position (Option<Pubkey>)
        4 * 2 +  // target_allocations [u16; 4]
        4 * 2 +  // current_allocations [u16; 4]
        1 + 32 + // oracle_price_feed (Option<Pubkey>)
        1 + 2 +  // liquidation_threshold_bps (Option<u16>)
        64;      // reserved
}

/// User position tracking (optional, for advanced features like leverage tracking per user)
#[account]
pub struct UserPosition {
    /// User wallet
    pub user: Pubkey,
    
    /// Vault this position belongs to
    pub vault: Pubkey,
    
    /// User's share balance
    pub shares: u64,
    
    /// User's leverage (for leveraged strategies, basis points)
    pub leverage_bps: u16,
    
    /// User's collateral amount
    pub collateral: u64,
    
    /// User's debt amount (for leveraged positions)
    pub debt: u64,
    
    /// Last interaction timestamp
    pub last_interaction: i64,
    
    /// Bump seed
    pub bump: u8,
}

impl UserPosition {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        32 + // vault
        8 +  // shares
        2 +  // leverage_bps
        8 +  // collateral
        8 +  // debt
        8 +  // last_interaction
        1;   // bump
}

/// Rebalance state (temporary account for rebalance operations)
#[account]
pub struct RebalanceState {
    /// Vault being rebalanced
    pub vault: Pubkey,
    
    /// Rebalancer (who initiated)
    pub rebalancer: Pubkey,
    
    /// Assets before rebalance
    pub assets_before: u64,
    
    /// Target allocation
    pub target_allocation: [u16; 4],
    
    /// Timestamp
    pub timestamp: i64,
    
    /// Bump seed
    pub bump: u8,
}

impl RebalanceState {
    pub const LEN: usize = 8 + // discriminator
        32 + // vault
        32 + // rebalancer
        8 +  // assets_before
        4 * 2 + // target_allocation
        8 +  // timestamp
        1;   // bump
}
