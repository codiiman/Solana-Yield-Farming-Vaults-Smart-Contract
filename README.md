# Solana Yield Farming Vaults

[![Anchor](https://img.shields.io/badge/Anchor-0.30.0-000000?logo=anchor)](https://www.anchor-lang.com/)
[![Solana](https://img.shields.io/badge/Solana-1.18+-9945FF?logo=solana)](https://solana.com/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-000000?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

> **Kamino Finance-inspired automated yield farming vaults smart contract for Solana**

A production-grade, modular Solana smart contract implementing automated yield farming vaults with features like auto-compounding, rebalancing, leveraged positions, and risk management. Inspired by Kamino Finance's Earn and Multiply vaults.

## 📋 Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Key Algorithms](#key-algorithms)
- [Installation](#installation)
- [Building](#building)
- [Testing](#testing)
- [Deployment](#deployment)
- [Usage Examples](#usage-examples)
- [Contact & Contribution](#contact)

## 🎯 Overview

This project implements a comprehensive yield farming vault system on Solana, allowing users to deposit assets and earn automated yields through various strategies:

- **LP Farming**: Automated liquidity provision to DEX pools (Raydium/Orca)
- **Leveraged Yield**: Amplified yields through borrowing from lending protocols
- **Auto-Compound**: Automatic reinvestment of rewards to maximize compounding
- **Delta-Neutral**: Hedged positions using perpetuals or options

The contract handles vault share minting, fee collection, position rebalancing, liquidation mechanisms, and integrates with oracles for price feeds.

## ✨ Features

### Core Functionality

- ✅ **Permissionless Vault Creation**: Create vaults with custom strategies and parameters
- ✅ **Deposit/Withdraw**: Users deposit assets and receive yield-bearing vault shares
- ✅ **Auto-Compounding**: Periodic harvest and reinvestment of rewards (permissionless)
- ✅ **Rebalancing**: Automated position adjustment based on market conditions
- ✅ **Leverage Support**: Multiply-style vaults with borrowing integration
- ✅ **Fee Management**: Performance and management fees collected to treasury
- ✅ **Oracle Integration**: Pyth price feeds for NAV calculation and liquidation checks
- ✅ **Liquidation System**: Automated liquidation of undercollateralized positions
- ✅ **Pause Mechanism**: Emergency pause/unpause functionality

### Security Features

- ✅ **Reentrancy Protection**: Anchor's built-in account validation
- ✅ **Overflow Protection**: Checked math operations throughout
- ✅ **Authority Validation**: Strict access control for sensitive operations
- ✅ **Oracle Freshness Checks**: Validates price feed staleness
- ✅ **Slippage Protection**: Configurable slippage tolerances
- ✅ **Health Factor Monitoring**: Real-time leverage and liquidation risk tracking

## 🏗️ Architecture

### Account Structure

```
GlobalState (Protocol-wide configuration)
├── authority: Protocol admin
├── treasury: Fee collection address
├── default_management_fee_bps: Default management fee (basis points)
├── default_performance_fee_bps: Default performance fee (basis points)
├── paused: Protocol pause flag
└── vault_count: Total vaults created

Vault (Individual vault instance)
├── vault_id: Unique identifier
├── strategy: Strategy type (0-3)
├── underlying_mint: Base asset (e.g., SOL, USDC)
├── share_mint: Vault share token mint
├── vault_token_account: Vault's asset holdings
├── total_assets: Total AUM (Assets Under Management)
├── total_shares: Total shares minted
├── fees: Management & performance fee configuration
├── leverage: Current and max leverage settings
├── strategy_config: Strategy-specific parameters
└── timestamps: Last harvest/rebalance times

UserPosition (Optional, for leveraged strategies)
├── user: User wallet
├── vault: Associated vault
├── shares: User's share balance
├── collateral: Collateral amount
└── debt: Borrowed amount
```

### Instruction Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Protocol Initialization                  │
│  initialize_global_state() → Set protocol defaults         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      Vault Creation                          │
│  initialize_vault() → Create vault with strategy config     │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    User Interactions                         │
│  deposit() → Mint shares, transfer assets                   │
│  withdraw() → Burn shares, return assets                    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  Automated Operations                        │
│  harvest() → Collect rewards, auto-compound                 │
│  rebalance() → Adjust positions to target allocations       │
│  liquidate() → Liquidate undercollateralized positions      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Fee Collection                            │
│  collect_fees() → Transfer fees to treasury                 │
└─────────────────────────────────────────────────────────────┘
```

### Strategy Types

1. **LP Farming (0)**: Provide liquidity to DEX pools, earn trading fees + rewards
2. **Leveraged Yield (1)**: Borrow to amplify position size, higher risk/reward
3. **Auto-Compound (2)**: Automatically reinvest rewards to maximize compounding
4. **Delta-Neutral (3)**: Hedged positions using perpetuals or options

## 🔢 Key Algorithms

### Yield Accrual Formula

**Shares Calculation (First Deposit)**:
```
shares = deposit_amount  // 1:1 ratio
```

**Shares Calculation (Subsequent Deposits)**:
```
shares = (deposit_amount × total_shares) / total_assets
```

**NAV (Net Asset Value) per Share**:
```
nav_per_share = total_assets / total_shares
```

**Assets to Withdraw**:
```
assets = (shares × total_assets) / total_shares
```

### Fee Calculations

**Management Fee** (accrued continuously):
```
fee = total_assets × management_fee_bps × time_elapsed / (10000 × seconds_per_year)
```

**Performance Fee** (on gains above high water mark):
```
fee = (current_nav - high_water_mark) × performance_fee_bps / 10000
```

### Leverage & Health Factor

**Position Size**:
```
position_size = collateral × leverage_bps / 10000
debt = position_size - collateral
```

**Health Factor**:
```
health_factor = (collateral × collateral_factor_bps) / debt
```

**Liquidation Trigger**:
```
if health_factor < liquidation_threshold_bps:
    liquidate()
```

### Rebalance Trigger

**Allocation Deviation Check**:
```
for each asset in allocations:
    deviation = |current_allocation - target_allocation|
    if deviation > rebalance_threshold_bps:
        trigger_rebalance()
```

### APY Estimation

**Simplified APY Calculation**:
```
apy_bps = (rewards_per_period / total_assets) × (seconds_per_year / period_seconds) × 10000
```

## 🚀 Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (1.18+)
- [Anchor Framework](https://www.anchor-lang.com/docs/installation) (0.30+)
- [Node.js](https://nodejs.org/) (18+) and [Yarn](https://yarnpkg.com/)

### Setup

1. **Clone the repository**:
```bash
git clone https://github.com/yourusername/solana-yield-farming-vaults.git
cd solana-yield-farming-vaults
```

2. **Install dependencies**:
```bash
# Install Anchor dependencies
anchor build

# Install Node.js dependencies
yarn install
```

3. **Configure Solana CLI**:
```bash
solana config set --url localhost
solana-keygen new  # If you don't have a keypair
```

## 🔨 Building

Build the Anchor program:

```bash
anchor build
```

This will:
- Compile the Rust program
- Generate the IDL (Interface Definition Language)
- Create the program binary

## 🧪 Testing

Run the test suite:

```bash
# Start local validator (in separate terminal)
solana-test-validator

# Run tests
anchor test
```

### Test Coverage

The test suite includes:

- ✅ Global state initialization
- ✅ Vault creation with different strategies
- ✅ Deposit/withdraw operations
- ✅ Share minting and burning
- ✅ Fee calculations
- ✅ Pause/unpause functionality
- ✅ Leverage adjustments (stub)
- ✅ Liquidation scenarios (stub)

## 📦 Deployment

### Local Deployment

1. **Build the program**:
```bash
anchor build
```

2. **Deploy to localnet**:
```bash
anchor deploy
```

### Devnet/Mainnet Deployment

1. **Update program ID** in `Anchor.toml` and `lib.rs`

2. **Build for target cluster**:
```bash
anchor build
```

3. **Deploy**:
```bash
# Set cluster
solana config set --url devnet  # or mainnet-beta

# Airdrop SOL (devnet only)
solana airdrop 2

# Deploy
anchor deploy
```

## 💻 Usage Examples

### Initialize Protocol

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";

const program = anchor.workspace.SolanaYieldFarmingVaults;

// Initialize global state
await program.methods
  .initializeGlobalState(
    new anchor.BN(200),   // 2% management fee
    new anchor.BN(2000)   // 20% performance fee
  )
  .accounts({
    globalState: globalStatePda,
    authority: wallet.publicKey,
    treasury: treasuryKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

### Create Vault

```typescript
// Create LP Farming vault
await program.methods
  .initializeVault(
    0,                    // Strategy: LP Farming
    null,                 // Use default management fee
    null,                 // Use default performance fee
    null,                 // No leverage (1x)
    new anchor.BN(1000000) // Min deposit: 0.001 tokens
  )
  .accounts({
    vault: vaultPda,
    globalState: globalStatePda,
    underlyingMint: usdcMint,
    shareMint: shareMint,
    vaultTokenAccount: vaultTokenAccount,
    authority: wallet.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

### Deposit Assets

```typescript
const depositAmount = new anchor.BN(1000000000); // 1 token

await program.methods
  .deposit(depositAmount)
  .accounts({
    vault: vaultPda,
    vaultTokenAccount: vaultTokenAccount,
    userTokenAccount: userTokenAccount,
    shareMint: shareMint,
    userShareAccount: userShareAccount,
    user: wallet.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

### Harvest Rewards

```typescript
// Anyone can call harvest (permissionless)
const rewardsAmount = new anchor.BN(50000000); // 0.05 tokens

await program.methods
  .harvest(rewardsAmount)
  .accounts({
    vault: vaultPda,
    globalState: globalStatePda,
    vaultTokenAccount: vaultTokenAccount,
    rewardsTokenAccount: rewardsTokenAccount,
    rewardsAuthority: rewardsAuthority,
    harvester: wallet.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

### Rebalance Positions

```typescript
const targetAllocations = [5000, 3000, 1500, 500]; // 50%, 30%, 15%, 5%

await program.methods
  .rebalance(targetAllocations)
  .accounts({
    vault: vaultPda,
    rebalancer: wallet.publicKey,
  })
  .rpc();
```

### Adjust Leverage

```typescript
const targetLeverage = new anchor.BN(20000); // 2x leverage
const collateralAdd = new anchor.BN(100000000); // 0.1 tokens

await program.methods
  .adjustLeverage(targetLeverage, collateralAdd)
  .accounts({
    vault: vaultPda,
    userPosition: userPositionPda,
    vaultTokenAccount: vaultTokenAccount,
    userTokenAccount: userTokenAccount,
    user: wallet.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

## 🙏 Acknowledgments

- Inspired by [Kamino Finance](https://www.kamino.finance/)'s innovative yield vault architecture
- Built with [Anchor Framework](https://www.anchor-lang.com/)
- Uses [Pyth Network](https://pyth.network/) for price feeds (stub)

## 📞 Contact

- telegram: https://t.me/codiiman
- twitter:  https://x.com/codiiman_
