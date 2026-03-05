import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaYieldFarmingVaults } from "../target/types/solana_yield_farming_vaults";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createMint,
  createAccount,
  mintTo,
} from "@solana/spl-token";
import { expect } from "chai";

describe("solana-yield-farming-vaults", () => {
  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaYieldFarmingVaults as Program<SolanaYieldFarmingVaults>;
  const wallet = provider.wallet;

  // Test accounts
  let globalState: anchor.web3.PublicKey;
  let treasury: anchor.web3.PublicKey;
  let underlyingMint: anchor.web3.PublicKey;
  let shareMint: anchor.web3.PublicKey;
  let vault: anchor.web3.PublicKey;
  let vaultTokenAccount: anchor.web3.PublicKey;
  let userTokenAccount: anchor.web3.PublicKey;
  let userShareAccount: anchor.web3.PublicKey;

  const [globalStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("global_state")],
    program.programId
  );

  before(async () => {
    // Initialize test mints and accounts
    treasury = wallet.publicKey;
    
    // Create underlying token mint
    underlyingMint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      9 // 9 decimals
    );

    // Create share token mint
    shareMint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      9
    );

    // Get vault PDA
    const [vaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), Buffer.from([0, 0, 0, 0, 0, 0, 0, 0])],
      program.programId
    );
    vault = vaultPda;

    // Get vault token account
    vaultTokenAccount = await getAssociatedTokenAddress(
      underlyingMint,
      vault,
      true
    );

    // Create user token account
    userTokenAccount = await getAssociatedTokenAddress(
      underlyingMint,
      wallet.publicKey
    );

    // Create user share account
    userShareAccount = await getAssociatedTokenAddress(
      shareMint,
      wallet.publicKey
    );
  });

  it("Initializes global state", async () => {
    const managementFeeBps = 200; // 2%
    const performanceFeeBps = 2000; // 20%

    const tx = await program.methods
      .initializeGlobalState(managementFeeBps, performanceFeeBps)
      .accounts({
        globalState: globalStatePda,
        authority: wallet.publicKey,
        treasury: treasury,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Global state initialized:", tx);

    const globalStateAccount = await program.account.globalState.fetch(globalStatePda);
    expect(globalStateAccount.authority.toString()).to.equal(wallet.publicKey.toString());
    expect(globalStateAccount.defaultManagementFeeBps).to.equal(managementFeeBps);
    expect(globalStateAccount.defaultPerformanceFeeBps).to.equal(performanceFeeBps);
    expect(globalStateAccount.paused).to.be.false;
  });

  it("Initializes a vault", async () => {
    const strategy = 0; // LP Farming
    const minDeposit = new anchor.BN(1000000); // 0.001 tokens (9 decimals)

    const tx = await program.methods
      .initializeVault(
        strategy,
        null, // Use default management fee
        null, // Use default performance fee
        null, // No leverage (1x)
        minDeposit
      )
      .accounts({
        vault: vault,
        globalState: globalStatePda,
        underlyingMint: underlyingMint,
        shareMint: shareMint,
        vaultTokenAccount: vaultTokenAccount,
        authority: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Vault initialized:", tx);

    const vaultAccount = await program.account.vault.fetch(vault);
    expect(vaultAccount.vaultId.toString()).to.equal("0");
    expect(vaultAccount.strategy).to.equal(strategy);
    expect(vaultAccount.totalAssets.toString()).to.equal("0");
    expect(vaultAccount.totalShares.toString()).to.equal("0");
    expect(vaultAccount.paused).to.be.false;
  });

  it("Deposits into vault", async () => {
    const depositAmount = new anchor.BN(1000000000); // 1 token (9 decimals)

    // Mint tokens to user
    await mintTo(
      provider.connection,
      wallet.payer,
      underlyingMint,
      userTokenAccount,
      wallet.publicKey,
      depositAmount.toNumber()
    );

    const tx = await program.methods
      .deposit(depositAmount)
      .accounts({
        vault: vault,
        vaultTokenAccount: vaultTokenAccount,
        userTokenAccount: userTokenAccount,
        shareMint: shareMint,
        userShareAccount: userShareAccount,
        user: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("Deposit transaction:", tx);

    const vaultAccount = await program.account.vault.fetch(vault);
    expect(vaultAccount.totalAssets.toString()).to.equal(depositAmount.toString());
    expect(vaultAccount.totalShares.toString()).to.equal(depositAmount.toString()); // First deposit = 1:1
  });

  it("Withdraws from vault", async () => {
    const vaultAccountBefore = await program.account.vault.fetch(vault);
    const sharesToBurn = vaultAccountBefore.totalShares.div(new anchor.BN(2)); // Withdraw 50%

    const tx = await program.methods
      .withdraw(sharesToBurn)
      .accounts({
        vault: vault,
        vaultTokenAccount: vaultTokenAccount,
        userTokenAccount: userTokenAccount,
        shareMint: shareMint,
        userShareAccount: userShareAccount,
        user: wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("Withdraw transaction:", tx);

    const vaultAccountAfter = await program.account.vault.fetch(vault);
    expect(vaultAccountAfter.totalAssets.toString()).to.equal(
      vaultAccountBefore.totalAssets.div(new anchor.BN(2)).toString()
    );
    expect(vaultAccountAfter.totalShares.toString()).to.equal(
      vaultAccountBefore.totalShares.sub(sharesToBurn).toString()
    );
  });

  it("Pauses and unpauses vault", async () => {
    // Pause
    const pauseTx = await program.methods
      .pauseVault()
      .accounts({
        vault: vault,
        authority: wallet.publicKey,
      })
      .rpc();

    console.log("Pause transaction:", pauseTx);

    let vaultAccount = await program.account.vault.fetch(vault);
    expect(vaultAccount.paused).to.be.true;

    // Try to deposit while paused (should fail)
    try {
      const depositAmount = new anchor.BN(1000000);
      await program.methods
        .deposit(depositAmount)
        .accounts({
          vault: vault,
          vaultTokenAccount: vaultTokenAccount,
          userTokenAccount: userTokenAccount,
          shareMint: shareMint,
          userShareAccount: userShareAccount,
          user: wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
      
      expect.fail("Deposit should have failed while vault is paused");
    } catch (err) {
      expect(err.toString()).to.include("VaultPaused");
    }

    // Unpause
    const unpauseTx = await program.methods
      .unpauseVault()
      .accounts({
        vault: vault,
        authority: wallet.publicKey,
      })
      .rpc();

    console.log("Unpause transaction:", unpauseTx);

    vaultAccount = await program.account.vault.fetch(vault);
    expect(vaultAccount.paused).to.be.false;
  });
});
