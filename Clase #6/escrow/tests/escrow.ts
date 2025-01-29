import { Escrow } from "../target/types/escrow";
import * as anchor from "@coral-xyz/anchor";
import { BN } from "bn.js";


describe("escrow", () => {

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as anchor.Program<Escrow>;

  before(async () => {


  });

  it("Usuario Bob inicializa un escrow", async() => {

    const tokenAAmount = new BN(100);
    const tokenBAmount = new BN(95);

    const tx = await program.methods.initializar(tokenAAmount,tokenBAmount)
  })

  it("Usuario Alice finaliza el escrow", async() => {

    
    const tx = await program.methods.finalizar()
  })

});