// No imports needed: web3, anchor, pg and more are globally available
import { PublicKey } from "@solana/web3.js";
import * as spl from "@solana/spl-token";
import * as web3 from "@solana/web3.js";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { assert } from "chai";
import { Program } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import { BN } from "bn.js";

describe("Test", () => {
  // antes de nada definimos las cuentas que vamos a necesitar
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ManejadorEventos as Program<Escrow>;

  // tokens
  let tokenA: PublicKey; // example: USDC
  let tokenB: PublicKey; // example: USDT

  // cuentas
  let escrow: PublicKey; // donde almacenamos la información del intercambio
  let garantia: PublicKey; // donde se almacenan los tokens A  del inicializador

  /* 
   El usuario inicializador que será nuestra wallet
   */
  let inicializador = provider.wallet as NodeWallet;
  let inicializadorTokenA: PublicKey; // cuenta token asociada al incializador y el Token A

  // id del scrow (timestamp)
  let id = Date.now().toString();

  /*
  creamos todas las cuentas necesarias que denben existir previamente
  a la ejecución de nuestra instruccion
   */
  before(async () => {
    // encontramos una dirección PDA para la cuenta del escrow
    // ESTO NO CREA LA CUENTA
    [escrow] = web3.PublicKey.findProgramAddressSync(
      [inicializador.publicKey.toBuffer(), Buffer.from(id)],
      program.programId
    );
    console.log("cuenta del escrow: ", escrow.toBase58());

    // encontramos una dirección PDA para la cuenta de garantía
    [garantia] = anchor.web3.PublicKey.findProgramAddressSync(
      [escrow.toBuffer()],
      program.programId
    );
    console.log("cuenta dela garantia: ", garantia.toBase58());

    // creamos el token A
    tokenA = await spl.createMint(
      provider.connection, // conexion a solana
      inicializador.payer, // el que paga los fees
      inicializador.publicKey, // el mint authority
      inicializador.publicKey, // el freeza authority
      2 // decimales del token
    );
    console.log("token A: ", tokenA.toBase58());

    // creamos el token B
    tokenB = await spl.createMint(
      provider.connection, // conexion a solana
      inicializador.payer, // el que paga los fees
      inicializador.publicKey, // el mint authority
      inicializador.publicKey, // el freeza authority
      2 // decimales del token
    );
    console.log("token B: ", tokenB.toBase58());

    // creamos la cuenta token asociada al inicializador y el token A
    inicializadorTokenA = await spl.createAssociatedTokenAccount(
      provider.connection, // conexion a la red
      inicializador.payer, // paga los fees
      tokenA, // tokens almacenados en la cuenta
      inicializador.publicKey // owner de los tokens
    );
    console.log("cuenta inicializadorTokenA: ", inicializadorTokenA.toBase58());

    /*
    nuestra primera isntrucción transfiere tokens a a la cuenta de garantia
    el inicializador debe poseer tokens A en su cuenta token asi que
    hacemos mint de tokens A a la cuenta token asociada al incializador y el token A
    */
    await spl.mintTo(
      provider.connection, // conexion a solana
      inicializador.payer, // el que paga los fees
      tokenA, // token a mintear
      inicializadorTokenA, // donde depositarlos
      inicializador.publicKey, // mint authority
      100000 // cantidad a mintear (expresada en decimales)
    );
  });

  //--------------------------- TESTS -----------------------//
  it("Se inicializa un Escrow", async () => {
    // montos asociados al intercambio de tokens
    const cantidadTokenA = new BN(100); // 100 tokens A
    const cantidadTokenB = new BN(95); // 95 tokens B

    // llamamos a la isntrucción
    let txHash = await program.methods
      .inicializar(id, cantidadTokenA, cantidadTokenB)
      .accounts({
        escrow: escrow,
        inicializador: inicializador.publicKey,
        inicializadorCuentaTokenA: inicializadorTokenA,
        cuentaDeGarantia: garantia,
        tokenA: tokenA,
        tokenB: tokenB,
      })
      .signers([inicializador.payer])
      .rpc();

    // Confirmamos la transaccion transaction
    await provider.connection.confirmTransaction(txHash);

    // verificamos que se haya depositado la cantidad en la cuenta de garantía
    let deposito = (await spl.getAccount(provider.connection, garantia)).amount;

    // assert
    assert.equal(cantidadTokenA.toNumber() * 10 ** 2, Number(deposito));
  });
});
