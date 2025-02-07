import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ManejadorEventos } from "../target/types/manejador_eventos";
import * as spl from "@solana/spl-token";
import * as web3 from "@solana/web3.js";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { assert } from "chai";


describe("manejador_eventos", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ManejadorEventos as Program<ManejadorEventos>;

  // declaro las cuentas necesarias
  let autoridad = provider.wallet as NodeWallet; // walet conectada a playground

  let tokenAceptado: web3.PublicKey; // example: USDC

  // PDAs
  let evento: web3.PublicKey;
  let tokenEvento: web3.PublicKey; // sponsorship token
  let bovedaEvento: web3.PublicKey;
  let bovedaGanancias: web3.PublicKey;

  // id del Evento
  let id: string = Date.now().toString();

  // creamos todo lo necesario antes de correr el test
  before(async () => {
    // buscamos la PDA del evento
    [evento] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from(id), Buffer.from("evento"), autoridad.publicKey.toBuffer()],
      program.programId
    );
    console.log("cuenta del evento: ", evento.toBase58());

    // PDA del token del evento
    [tokenEvento] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("token_evento"), evento.toBuffer()],
      program.programId
    );
    console.log("cuenta del token del evento: ", tokenEvento.toBase58());

    // PDA boveda del evento
    [bovedaEvento] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("boveda_evento"), evento.toBuffer()],
      program.programId
    );
    console.log("cuenta de la boveda del evento: ", bovedaEvento.toBase58());

    // PDA boveda de ganacias
    [bovedaGanancias] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("boveda_ganancias"), evento.toBuffer()],
      program.programId
    );
    console.log(
      "cuenta de la boveda de ganancias: ",
      bovedaGanancias.toBase58()
    );

    // creamos el mint del token aceptado (para comprar entradas y tokens)
    tokenAceptado = await spl.createMint(
      provider.connection, // conexion a solana
      autoridad.payer, // el que paga los fees
      autoridad.publicKey, // el mint authority
      autoridad.publicKey, // el freeza authority
      2 // decimales del token
    );
  });

  it("Crear un evento", async () => {
    // Datos basicos del evento
    const nombre: string = "Mi primer evento";
    const descripcion = "El mejor evento del mundo!";
    const precioEntrada = 2.1;
    const precioToken = 5.0;

    //llamamos a la instruccion del programa
    const tx = await program.methods
      .crearEvento(id, nombre, descripcion, precioEntrada, precioToken)
      .accounts({
        evento: evento,
        tokenAceptado: tokenAceptado,
        tokenEvento: tokenEvento,
        bovedaEvento: bovedaEvento,
        bovedaGanancias: bovedaGanancias,
        autoridad: autoridad.publicKey,
      })
      .rpc();

    //Confirmamos la transaccion
    await provider.connection.confirmTransaction(tx);

    //Podemos ver la informacion almacenada en la cuenta del evento
    const infoEvento = await program.account.evento.fetch(evento);

    console.log("Información del evento: ", infoEvento);

    // con al informacion del evento podemos hacer comprobaciones
    // comprobamos que el precio del token sea correcto (y esta expresado en la unidad minima del token)
    assert.equal(infoEvento.precioToken.toNumber(), precioToken );
  });

  it("Finaliza un evento", async () => {
    // llamamo a la instrucción eliminar
    const tx = await program.methods
      .finalizarEvento()
      // enviamos las cuentas asociadas a la instrucción
      .accounts({
        evento: evento,
        autoridad: autoridad.publicKey,
      })
      // firma la autoridad creadora del evento
      .signers([autoridad.payer])
      // enviamos a la red
      .rpc();

    //Confirmamos la transaccion
    const latestBlockHash = await provider.connection.getLatestBlockhash();

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: tx,
    });

    //Podemos ver la informacion almacenada en la cuenta del evento
    // en este caso debe ser null porque no debe existir
    const infoEvento = await program.account.evento.fetchNullable(evento);

    console.log("Evento activo: ", infoEvento.activo);
  });
  it("Elimina el evento creado anteriormente", async () => {
    // llamamo a la instrucción eliminar
    const tx = await program.methods
      .eliminarEvento()
      // enviamos las cuentas asociadas a la instrucción
      .accounts({
        evento: evento,
        bovedaEvento: bovedaEvento,
        bovedaGanancias: bovedaGanancias,
        tokenEvento: tokenEvento,
        autoridad: autoridad.publicKey,
      })
      // firma la autoridad creadora del evento
      .signers([autoridad.payer])
      // enviamos a la red
      .rpc();
    //Confirmamos la transaccion
    //Confirmamos la transaccion
    const latestBlockHash = await provider.connection.getLatestBlockhash();

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: tx,
    });


    //Podemos ver la informacion almacenada en la cuenta del evento
    // en este caso debe ser null porque no debe existir
    const infoEvento = await program.account.evento.fetchNullable(evento);

    console.log("Información del evento: ", infoEvento);
  });

});
