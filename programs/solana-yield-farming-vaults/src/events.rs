use anchor_lang::prelude::*;

/// Event emitted when a vault is initialized
#[event]
pub struct VaultInitialized {
    pub vault: Pubkey,
    pub strategy: u8,
    pub underlying_mint: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when a user deposits into a vault
#[event]
pub struct DepositEvent {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub shares_minted: u64,
    pub total_assets: u64,
    pub total_shares: u64,
    pub timestamp: i64,
}

/// Event emitted when a user withdraws from a vault
#[event]
pub struct WithdrawEvent {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub shares_burned: u64,
    pub total_assets: u64,
    pub total_shares: u64,
    pub timestamp: i64,
}

/// Event emitted when rewards are harvested and compounded
#[event]
pub struct HarvestEvent {
    pub vault: Pubkey,
    pub harvester: Pubkey,
    pub rewards_harvested: u64,
    pub rewards_reinvested: u64,
    pub new_total_assets: u64,
    pub apy_estimate: u64, // Basis points (10000 = 100%)
    pub timestamp: i64,
}

/// Event emitted when a vault position is rebalanced
#[event]
pub struct RebalanceEvent {
    pub vault: Pubkey,
    pub rebalancer: Pubkey,
    pub assets_before: u64,
    pub assets_after: u64,
    pub rebalance_type: u8, // 0 = full, 1 = partial
    pub price_before: i64,
    pub price_after: i64,
    pub timestamp: i64,
}

/// Event emitted when a leveraged position is liquidated
#[event]
pub struct LiquidationEvent {
    pub vault: Pubkey,
    pub liquidator: Pubkey,
    pub liquidated_user: Pubkey,
    pub collateral_seized: u64,
    pub debt_repaid: u64,
    pub health_factor_before: u64, // Basis points
    pub timestamp: i64,
}

/// Event emitted when fees are collected
#[event]
pub struct FeeCollectionEvent {
    pub vault: Pubkey,
    pub management_fee: u64,
    pub performance_fee: u64,
    pub total_fees: u64,
    pub treasury: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when leverage is adjusted
#[event]
pub struct LeverageAdjustmentEvent {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub leverage_before: u64, // Basis points (10000 = 1x)
    pub leverage_after: u64,
    pub collateral_added: u64,
    pub debt_added: u64,
    pub timestamp: i64,
}
