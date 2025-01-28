import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { CpiPda } from "../target/types/cpi_pda";
import {
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";

describe("cpi", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CpiPda as Program<CpiPda>;

  const connection = program.provider.connection;

  const wallet = provider.wallet as anchor.Wallet;
  const [PDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("pda")],
    program.programId
  );

  const transferAmount = 0.1 * LAMPORTS_PER_SOL;

  it("Fund PDA with SOL", async () => {
    const transferInstruction = SystemProgram.transfer({
      fromPubkey: wallet.publicKey,
      toPubkey: PDA,
      lamports: transferAmount,
    });

    const transaction = new Transaction().add(transferInstruction);

    const transactionSignature = await sendAndConfirmTransaction(
      connection,
      transaction,
      [wallet.payer] // signer
    );

    console.log(
      `\nTransaction Signature: https://solana.fm/tx/${transactionSignature}?cluster=devnet-solana`
    );
  });

  it("SOL Transfer with PDA signer", async () => {
    const transactionSignature = await program.methods
      .transferencia(new BN(transferAmount))
      .accounts({
        pdaRemitente: PDA,
        recipiente: wallet.publicKey,
      })
      .rpc();

    console.log(
      `\nTransaction Signature: https://solana.fm/tx/${transactionSignature}?cluster=devnet-solana`
    );
  });
});
