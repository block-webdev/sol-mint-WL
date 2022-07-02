import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SolMintWl } from "../target/types/sol_mint_wl";

describe("sol-mint-WL", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SolMintWl as Program<SolMintWl>;

  it("Is initialized!", async () => {
    // Add your test here.
    // const tx = await program.methods.initialize().rpc();
    // console.log("Your transaction signature", tx);

    let str = String("hello");// .fromCharCode([97, 98]);
    console.log('input str', str);

    const tx1 = await program.methods.strTest(str).rpc();
    console.log("Your transaction signature", tx1);
  });
});
