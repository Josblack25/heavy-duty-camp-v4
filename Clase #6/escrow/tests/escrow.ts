import { Escrow } from "../target/types/escrow";
import * as anchor from "@coral-xyz/anchor";
import { BN } from "bn.js";
import { PublicKey } from '@solana/web3.js';
import * as spl from '@solana/spl-token';
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import assert from "assert";

describe("escrow", () => {

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as anchor.Program<Escrow>;

  // tokens
   let tokenA: PublicKey; // example: USDC
   let tokenB: PublicKey; // example: USDT

   // cuentas
   let escrow: PublicKey; // donde almacenamos la información del intercambnio
   let garantia: PublicKey; // donde se almacenan los tokens del inicializador

   /* 
   El usuario inicializador que será nuestra wallet
   */
   let inicializador = provider.wallet as NodeWallet; 

   let inicializadorTokenA: PublicKey; // cuenta token asociada al incializador y el Token A

  before(async () => {
    // todo lo encesario antes de correr los test

    // cuenta del incializador
     console.log("pk inicializador: ", inicializador.publicKey);

     // encontramos una dirección PDa para la cuenta del escrow
     [escrow] = anchor.web3.PublicKey.findProgramAddressSync(
      [inicializador.publicKey.toBuffer()], program.programId);

    console.log("pk escrow: ", escrow);

     // encontramos una dirección PDA para la cuenta de garantía
     [garantia] = anchor.web3.PublicKey.findProgramAddressSync(
      [escrow.toBuffer()],
      program.programId
    );

    console.log("pk garantia: ", garantia);

    // creamos el token A
     tokenA = await spl.createMint(provider.connection, inicializador.payer, inicializador.publicKey, inicializador.publicKey, 2);
     console.log("pk tokenA: ", tokenA);

    // creamos la cuenta token asociada al inicializador y el token A
     inicializadorTokenA = await spl.createAssociatedTokenAccount(provider.connection,inicializador.payer,tokenA,inicializador.publicKey);
     console.log("pk inicializadorTokenA: ", inicializadorTokenA);

    // creamos el token B
     tokenB = await spl.createMint(provider.connection, inicializador.payer, inicializador.publicKey, inicializador.publicKey, 2);
     console.log("pk tokenB: ", tokenB);

    // hacemos mint de tokens A a la cuenta token asociada al incializador y el token A
     await spl.mintTo(provider.connection, inicializador.payer,tokenA,inicializadorTokenA,inicializador.payer, 100000);

  });

  it("Usuario inicializa un escrow", async() => {

    const tokenAAmount = new BN(100);
    const tokenBAmount = new BN(95);

    const tx = await program.methods.initializar(tokenAAmount,tokenBAmount).accounts({
      escrow: escrow,
      inicializador: inicializador.publicKey,
      inicializadorTokenAccountA: inicializadorTokenA,
      cuentaDeGarantia: garantia,
      mintA: tokenA,
      mintB: tokenB
    }).signers([inicializador.payer]).rpc();

    const escrowAccount = await program.account.escrow.fetch(escrow);
    console.log("Cuenta escrow: ", escrowAccount);

    assert.equal(100, (await spl.getAccount(provider.connection, garantia)).amount);
  });

});