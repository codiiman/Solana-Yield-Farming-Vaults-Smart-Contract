use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use crate::state::*;
use crate::errors::VaultError;

/// Initialize the global protocol state
pub fn initialize_global_state(
    ctx: Context<InitializeGlobalState>,
    management_fee_bps: u16,
    performance_fee_bps: u16,
) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;
    
    // Validate fees (max 10% management, 50% performance)
    require!(
        management_fee_bps <= 1000,
        VaultError::InvalidFeeConfig
    );
    require!(
        performance_fee_bps <= 5000,
        VaultError::InvalidFeeConfig
    );
    
    global_state.authority = ctx.accounts.authority.key();
    global_state.treasury = ctx.accounts.treasury.key();
    global_state.default_management_fee_bps = management_fee_bps;
    global_state.default_performance_fee_bps = performance_fee_bps;
    global_state.paused = false;
    global_state.vault_count = 0;
    global_state.bump = ctx.bumps.global_state;
    
    msg!("Global state initialized with fees: {} bps management, {} bps performance", 
         management_fee_bps, performance_fee_bps);
    
    Ok(())
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
    let global_state = &mut ctx.accounts.global_state;
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;
    
    // Validate strategy type (0-3)
    require!(
        strategy <= 3,
        VaultError::InvalidStrategy
    );
    
    // Validate leverage (max 5x = 50000 bps)
    if let Some(leverage) = max_leverage_bps {
        require!(
            leverage <= 50000 && leverage >= 10000,
            VaultError::InvalidLeverage
        );
    }
    
    // Get fees from global defaults or provided values
    let mgmt_fee = management_fee_bps.unwrap_or(global_state.default_management_fee_bps);
    let perf_fee = performance_fee_bps.unwrap_or(global_state.default_performance_fee_bps);
    
    require!(
        mgmt_fee <= 1000 && perf_fee <= 5000,
        VaultError::InvalidFeeConfig
    );
    
    // Initialize vault
    vault.vault_id = global_state.vault_count;
    vault.strategy = strategy;
    vault.underlying_mint = ctx.accounts.underlying_mint.key();
    vault.share_mint = ctx.accounts.share_mint.key();
    vault.vault_token_account = ctx.accounts.vault_token_account.key();
    vault.authority = ctx.accounts.authority.key();
    vault.total_assets = 0;
    vault.total_shares = 0;
    vault.last_harvest = clock.unix_timestamp;
    vault.last_rebalance = clock.unix_timestamp;
    vault.management_fee_bps = mgmt_fee;
    vault.performance_fee_bps = perf_fee;
    vault.high_water_mark = 0;
    vault.accrued_management_fees = 0;
    vault.accrued_performance_fees = 0;
    vault.paused = false;
    vault.min_deposit = min_deposit;
    vault.max_leverage_bps = max_leverage_bps.unwrap_or(10000); // 1x default
    vault.current_leverage_bps = 10000; // Start at 1x
    vault.rebalance_threshold_bps = 500; // 5% deviation threshold
    vault.harvest_cooldown = 3600; // 1 hour default
    vault.rebalance_cooldown = 86400; // 24 hours default
    vault.strategy_config = StrategyConfig::default();
    vault.bump = ctx.bumps.vault;
    
    // Increment vault count
    global_state.vault_count = global_state.vault_count
        .checked_add(1)
        .ok_or(VaultError::MathOverflow)?;
    
    emit!(VaultInitialized {
        vault: vault.key(),
        strategy,
        underlying_mint: vault.underlying_mint,
        authority: vault.authority,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Vault {} initialized with strategy {}", vault.vault_id, strategy);
    
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeGlobalState<'info> {
    #[account(
        init,
        payer = authority,
        space = GlobalState::LEN,
        seeds = [b"global_state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// CHECK: Treasury account (can be any address)
    pub treasury: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = authority,
        space = Vault::LEN,
        seeds = [b"vault", global_state.vault_count.to_le_bytes().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
    
    /// CHECK: Underlying token mint
    pub underlying_mint: Account<'info, Mint>,
    
    /// CHECK: Share token mint (should be initialized separately)
    pub share_mint: Account<'info, Mint>,
    
    /// CHECK: Vault's token account (should be initialized separately)
    pub vault_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
