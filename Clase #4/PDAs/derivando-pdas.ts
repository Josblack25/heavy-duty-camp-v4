import { PublicKey } from "@solana/web3.js";

const programId = new PublicKey("11111111111111111111111111111111");
// semilal opcional
const string = "helloWorld";

/*
const [PDA, bump] = PublicKey.findProgramAddressSync([], programId);
*/

const [PDA, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from(string)], // lista de semillas opcionales
  programId
);

console.log(`PDA: ${PDA}`);
console.log(`Bump: ${bump}`);
