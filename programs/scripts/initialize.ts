/**
 * SolanaVault Program Initialization Script
 *
 * This script initializes all deployed programs with proper configuration.
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import * as fs from "fs";

// Load deployed program IDs
const CLUSTER = process.env.CLUSTER || "devnet";
const deployedPath = `./deployed-${CLUSTER}.json`;

interface DeployedPrograms {
  vault_token: string;
  vault_staking: string;
  vault_rewards: string;
  vault_governance: string;
}

async function main() {
  console.log(`Initializing SolanaVault programs on ${CLUSTER}...`);

  // Load provider
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const wallet = provider.wallet as anchor.Wallet;

  console.log(`Wallet: ${wallet.publicKey.toBase58()}`);

  // Load deployed program IDs
  if (!fs.existsSync(deployedPath)) {
    throw new Error(`Deployed programs file not found: ${deployedPath}`);
  }

  const deployed: DeployedPrograms = JSON.parse(
    fs.readFileSync(deployedPath, "utf-8")
  );

  console.log("\nDeployed Program IDs:");
  console.log(`  vault_token: ${deployed.vault_token}`);
  console.log(`  vault_staking: ${deployed.vault_staking}`);
  console.log(`  vault_rewards: ${deployed.vault_rewards}`);
  console.log(`  vault_governance: ${deployed.vault_governance}`);

  // Load IDLs
  const tokenIdl = JSON.parse(
    fs.readFileSync("./target/idl/vault_token.json", "utf-8")
  );
  const stakingIdl = JSON.parse(
    fs.readFileSync("./target/idl/vault_staking.json", "utf-8")
  );
  const rewardsIdl = JSON.parse(
    fs.readFileSync("./target/idl/vault_rewards.json", "utf-8")
  );
  const governanceIdl = JSON.parse(
    fs.readFileSync("./target/idl/vault_governance.json", "utf-8")
  );

  // Create program instances
  const tokenProgram = new Program(
    tokenIdl,
    new PublicKey(deployed.vault_token),
    provider
  );
  const stakingProgram = new Program(
    stakingIdl,
    new PublicKey(deployed.vault_staking),
    provider
  );
  const rewardsProgram = new Program(
    rewardsIdl,
    new PublicKey(deployed.vault_rewards),
    provider
  );
  const governanceProgram = new Program(
    governanceIdl,
    new PublicKey(deployed.vault_governance),
    provider
  );

  // =========================================================================
  // 1. Initialize Token Program
  // =========================================================================
  console.log("\n1. Initializing Token Program...");

  const [tokenConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("token_config")],
    tokenProgram.programId
  );

  const mintKeypair = Keypair.generate();

  try {
    await tokenProgram.methods
      .initialize(new PublicKey(deployed.vault_rewards)) // Rewards program as emission authority
      .accounts({
        authority: wallet.publicKey,
        mint: mintKeypair.publicKey,
        tokenConfig: tokenConfig,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([mintKeypair])
      .rpc();

    console.log(`  Token Config: ${tokenConfig.toBase58()}`);
    console.log(`  Mint: ${mintKeypair.publicKey.toBase58()}`);
  } catch (e: any) {
    if (e.message?.includes("already in use")) {
      console.log("  Token program already initialized");
    } else {
      throw e;
    }
  }

  // =========================================================================
  // 2. Initialize Staking Program
  // =========================================================================
  console.log("\n2. Initializing Staking Program...");

  const [stakingPool] = PublicKey.findProgramAddressSync(
    [Buffer.from("staking_pool"), mintKeypair.publicKey.toBuffer()],
    stakingProgram.programId
  );

  const stakingVaultKeypair = Keypair.generate();

  try {
    await stakingProgram.methods
      .initialize(
        new PublicKey(deployed.vault_rewards), // Rewards authority
        new PublicKey(deployed.vault_rewards) // Slashing authority (same for now)
      )
      .accounts({
        authority: wallet.publicKey,
        vaultMint: mintKeypair.publicKey,
        stakingVault: stakingVaultKeypair.publicKey,
        stakingPool: stakingPool,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([stakingVaultKeypair])
      .rpc();

    console.log(`  Staking Pool: ${stakingPool.toBase58()}`);
    console.log(`  Staking Vault: ${stakingVaultKeypair.publicKey.toBase58()}`);
  } catch (e: any) {
    if (e.message?.includes("already in use")) {
      console.log("  Staking program already initialized");
    } else {
      throw e;
    }
  }

  // =========================================================================
  // 3. Initialize Rewards Program
  // =========================================================================
  console.log("\n3. Initializing Rewards Program...");

  const [rewardsConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("rewards_config")],
    rewardsProgram.programId
  );

  const rewardsVaultKeypair = Keypair.generate();
  const networkFundKeypair = Keypair.generate();

  // Create network fund token account first
  const networkFundAta = await getAssociatedTokenAddress(
    mintKeypair.publicKey,
    networkFundKeypair.publicKey
  );

  try {
    await rewardsProgram.methods
      .initialize(
        new PublicKey(deployed.vault_staking),
        new PublicKey(deployed.vault_token)
      )
      .accounts({
        authority: wallet.publicKey,
        vaultMint: mintKeypair.publicKey,
        rewardsVault: rewardsVaultKeypair.publicKey,
        networkFund: networkFundAta,
        rewardsConfig: rewardsConfig,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([rewardsVaultKeypair])
      .rpc();

    console.log(`  Rewards Config: ${rewardsConfig.toBase58()}`);
    console.log(`  Rewards Vault: ${rewardsVaultKeypair.publicKey.toBase58()}`);
    console.log(`  Network Fund: ${networkFundAta.toBase58()}`);
  } catch (e: any) {
    if (e.message?.includes("already in use")) {
      console.log("  Rewards program already initialized");
    } else {
      throw e;
    }
  }

  // =========================================================================
  // 4. Initialize Governance Program
  // =========================================================================
  console.log("\n4. Initializing Governance Program...");

  const [governanceConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("governance_config")],
    governanceProgram.programId
  );

  try {
    await governanceProgram.methods
      .initialize(new PublicKey(deployed.vault_staking))
      .accounts({
        authority: wallet.publicKey,
        governanceConfig: governanceConfig,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log(`  Governance Config: ${governanceConfig.toBase58()}`);
  } catch (e: any) {
    if (e.message?.includes("already in use")) {
      console.log("  Governance program already initialized");
    } else {
      throw e;
    }
  }

  // =========================================================================
  // Save Configuration
  // =========================================================================
  console.log("\n5. Saving configuration...");

  const config = {
    cluster: CLUSTER,
    programs: deployed,
    accounts: {
      tokenConfig: tokenConfig.toBase58(),
      mint: mintKeypair.publicKey.toBase58(),
      stakingPool: stakingPool.toBase58(),
      rewardsConfig: rewardsConfig.toBase58(),
      governanceConfig: governanceConfig.toBase58(),
    },
    authority: wallet.publicKey.toBase58(),
    timestamp: new Date().toISOString(),
  };

  const configPath = `./config-${CLUSTER}.json`;
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  console.log(`  Configuration saved to ${configPath}`);

  console.log("\n✅ Initialization complete!");
  console.log("\nNext steps:");
  console.log("  1. Fund the rewards vault with VAULT tokens");
  console.log("  2. Start the epoch advancement cron job");
  console.log("  3. Connect gateway nodes to record fees");
}

main().catch(console.error);
