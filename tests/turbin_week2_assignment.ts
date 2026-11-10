import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TurbinWeek2Assignment } from "../target/types/turbin_week2_assignment";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";

describe("turbin_week2_assignment", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.turbinWeek2Assignment as Program<TurbinWeek2Assignment>;

  const owner = (anchor.getProvider() as anchor.AnchorProvider).wallet;

  it("initializes, deposits, withdraws, and closes a SOL vault", async () => {
    const [vault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), owner.publicKey.toBuffer()],
      program.programId,
    );
    await program.methods.initializeVault().accounts({ owner: owner.publicKey, vault }).rpc();
    await program.methods.deposit(new anchor.BN(1_000_000)).accounts({ owner: owner.publicKey, vault }).rpc();
    await program.methods.withdraw(new anchor.BN(500_000)).accounts({ owner: owner.publicKey, recipient: owner.publicKey, vault }).rpc();
    await program.methods.closeVault().accounts({ owner: owner.publicKey, recipient: owner.publicKey, vault }).rpc();
    const closed = await anchor.getProvider().connection.getAccountInfo(vault);
    if (closed !== null) throw new Error("vault should be closed");
  });

  it("rejects zero deposits", async () => {
    const [vault] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("vault"), owner.publicKey.toBuffer()], program.programId);
    await program.methods.initializeVault().accounts({ owner: owner.publicKey, vault }).rpc();
    try {
      await program.methods.deposit(new anchor.BN(0)).accounts({ owner: owner.publicKey, vault }).rpc();
      throw new Error("expected zero amount to fail");
    } catch (error) {
      if (!(error as Error).message.includes("InvalidAmount")) throw error;
    }
  });

  it("makes, updates, and takes an SPL token escrow", async () => {
    const provider = anchor.getProvider() as anchor.AnchorProvider;
    const maker = anchor.web3.Keypair.generate();
    const taker = anchor.web3.Keypair.generate();
    for (const user of [maker, taker]) {
      const sig = await provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
      await provider.connection.confirmTransaction(sig, "confirmed");
    }
    const mintA = await createMint(provider.connection, maker, maker.publicKey, null, 6);
    const mintB = await createMint(provider.connection, maker, maker.publicKey, null, 6);
    const makerA = await getOrCreateAssociatedTokenAccount(provider.connection, maker, mintA, maker.publicKey);
    const makerB = await getOrCreateAssociatedTokenAccount(provider.connection, maker, mintB, maker.publicKey);
    const takerA = await getOrCreateAssociatedTokenAccount(provider.connection, taker, mintA, taker.publicKey);
    const takerB = await getOrCreateAssociatedTokenAccount(provider.connection, taker, mintB, taker.publicKey);
    await mintTo(provider.connection, maker, mintA, makerA.address, maker, 1_000_000);
    await mintTo(provider.connection, maker, mintB, takerB.address, maker, 2_000_000);
    const seed = new anchor.BN(Date.now());
    const [escrow] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("escrow"), maker.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)], program.programId);
    const vault = anchor.web3.Keypair.generate();
    const expiry = new anchor.BN(Math.floor(Date.now() / 1000) + 300);
    await program.methods.make(seed, new anchor.BN(600_000), new anchor.BN(1_000_000), expiry).accounts({ maker: maker.publicKey, mintA, mintB, makerTokenA: makerA.address, escrow, vault: vault.publicKey }).signers([maker, vault]).rpc();
    await program.methods.update(new anchor.BN(1_000_000), expiry).accounts({ maker: maker.publicKey, escrow }).signers([maker]).rpc();
    await program.methods.take().accounts({ taker: taker.publicKey, maker: maker.publicKey, mintA, mintB, takerTokenB: takerB.address, makerTokenB: makerB.address, takerTokenA: takerA.address, escrow, vault: vault.publicKey }).signers([taker]).rpc();
    const received = await provider.connection.getTokenAccountBalance(takerA.address);
    if (received.value.amount !== "600000") throw new Error("taker did not receive escrowed tokens");
  });

  it("makes and refunds an expired SPL token escrow", async () => {
    const provider = anchor.getProvider() as anchor.AnchorProvider;
    const maker = anchor.web3.Keypair.generate();
    const sig = await provider.connection.requestAirdrop(maker.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(sig, "confirmed");
    const mintA = await createMint(provider.connection, maker, maker.publicKey, null, 6);
    const mintB = await createMint(provider.connection, maker, maker.publicKey, null, 6);
    const makerA = await getOrCreateAssociatedTokenAccount(provider.connection, maker, mintA, maker.publicKey);
    await mintTo(provider.connection, maker, mintA, makerA.address, maker, 500_000);
    const seed = new anchor.BN(Date.now() + 1);
    const [escrow] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("escrow"), maker.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)], program.programId);
    const vault = anchor.web3.Keypair.generate();
    const expiry = new anchor.BN(Math.floor(Date.now() / 1000) + 2);
    await program.methods.make(seed, new anchor.BN(500_000), new anchor.BN(10), expiry).accounts({ maker: maker.publicKey, mintA, mintB, makerTokenA: makerA.address, escrow, vault: vault.publicKey }).signers([maker, vault]).rpc();
    await new Promise((resolve) => setTimeout(resolve, 2500));
    await program.methods.refund().accounts({ maker: maker.publicKey, mintA, makerTokenA: makerA.address, escrow, vault: vault.publicKey }).signers([maker]).rpc();
    const refunded = await provider.connection.getTokenAccountBalance(makerA.address);
    if (refunded.value.amount !== "500000") throw new Error("maker was not refunded");
  });
});
