use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod state;
pub mod utils;
pub mod instructions;

use instructions::*;

declare_id!("YvLtV8X9JzKpQmN3RfH5W2B6C4D7E8F9G0");

#[program]
pub mod solana_yield_farming_vaults {
    use super::*;

    /// Initialize the global protocol state
    pub fn initialize_global_state(
        ctx: Context<InitializeGlobalState>,
        management_fee_bps: u16,
        performance_fee_bps: u16,
    ) -> Result<()> {
        instructions::initialize::initialize_global_state(ctx, management_fee_bps, performance_fee_bps)
    }

    /// Initialize a new vault
    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        strategy: u8,
        management_fee_bps: Option<u16>,
        performance_fee_bps: Option<u16>,
        max_leverage_bps: Option<u16>,
        min_deposit: u64,
    ) -> Result<()> {
        instructions::initialize::initialize_vault(
            ctx,
            strategy,
            management_fee_bps,
            performance_fee_bps,
            max_leverage_bps,
            min_deposit,
        )
    }

    /// Deposit assets into a vault
    pub fn deposit(
        ctx: Context<Deposit>,
        amount: u64,
    ) -> Result<()> {
        instructions::deposit::deposit(ctx, amount)
    }

    /// Withdraw assets from a vault
    pub fn withdraw(
        ctx: Context<Withdraw>,
        shares: u64,
    ) -> Result<()> {
        instructions::withdraw::withdraw(ctx, shares)
    }

    /// Harvest rewards and auto-compound
    pub fn harvest(
        ctx: Context<Harvest>,
        rewards_amount: u64,
    ) -> Result<()> {
        instructions::harvest::harvest(ctx, rewards_amount)
    }

    /// Collect accrued fees to treasury
    pub fn collect_fees(
        ctx: Context<CollectFees>,
    ) -> Result<()> {
        instructions::harvest::collect_fees(ctx)
    }

    /// Rebalance vault positions
    pub fn rebalance(
        ctx: Context<Rebalance>,
        target_allocations: [u16; 4],
    ) -> Result<()> {
        instructions::rebalance::rebalance(ctx, target_allocations)
    }

    /// Update vault strategy configuration
    pub fn update_strategy_config(
        ctx: Context<UpdateStrategyConfig>,
        strategy_config: StrategyConfig,
    ) -> Result<()> {
        instructions::rebalance::update_strategy_config(ctx, strategy_config)
    }

    /// Update vault parameters
    pub fn update_vault_params(
        ctx: Context<UpdateVaultParams>,
        management_fee_bps: Option<u16>,
        performance_fee_bps: Option<u16>,
        harvest_cooldown: Option<i64>,
        rebalance_cooldown: Option<i64>,
        rebalance_threshold_bps: Option<u16>,
        min_deposit: Option<u64>,
    ) -> Result<()> {
        instructions::rebalance::update_vault_params(
            ctx,
            management_fee_bps,
            performance_fee_bps,
            harvest_cooldown,
            rebalance_cooldown,
            rebalance_threshold_bps,
            min_deposit,
        )
    }

    /// Liquidate an undercollateralized position
    pub fn liquidate(
        ctx: Context<Liquidate>,
        collateral_to_seize: u64,
    ) -> Result<()> {
        instructions::liquidate::liquidate(ctx, collateral_to_seize)
    }

    /// Adjust leverage for a leveraged position
    pub fn adjust_leverage(
        ctx: Context<AdjustLeverage>,
        target_leverage_bps: u16,
        collateral_add: u64,
    ) -> Result<()> {
        instructions::liquidate::adjust_leverage(ctx, target_leverage_bps, collateral_add)
    }

    /// Pause a vault
    pub fn pause_vault(
        ctx: Context<PauseVault>,
    ) -> Result<()> {
        instructions::pause::pause_vault(ctx)
    }

    /// Unpause a vault
    pub fn unpause_vault(
        ctx: Context<UnpauseVault>,
    ) -> Result<()> {
        instructions::pause::unpause_vault(ctx)
    }
}
