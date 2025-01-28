import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { Cpi } from "../target/types/cpi";
import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";

describe("cpi", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Cpi as Program<Cpi>;
  const sender = provider.wallet as anchor.Wallet;
  const recipient = new Keypair();

  const transferAmount = 0.01 * LAMPORTS_PER_SOL;

  it("SOL Transfer Anchor", async () => {
    const transactionSignature = await program.methods
      .transferir(new BN(transferAmount))
      .accounts({
        remitente: sender.publicKey,
        recipiente: recipient.publicKey,
      })
      .rpc();

    console.log(
      `\nTransaction Signature: https://solana.fm/tx/${transactionSignature}?cluster=devnet-solana`
    );
  });
});
