import * as anchor from "@coral-xyz/anchor";
import { Board } from "../target/types/board";

describe("board", () => {
  // Configure the client to use the local cluster.
  let provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.Board as anchor.Program<Board>

  //it("Is initialized!", async () => {
  //  const boardKeypair = anchor.web3.Keypair.generate()
  //  const owner = provider.wallet

  //  const tx = await program.methods
  //    .createBoard({
  //      rentalPrice: 1,
  //      width: 1024,
  //      height: 1024,
  //    })
  //    .accounts({
  //      owner: owner.publicKey,
  //      board: boardKeypair.publicKey
  //    })
  //    .signers([boardKeypair])
  //    .rpc()
  //  console.log("Your transaction signature", tx)
  //})

  it("Adds cell to pad", async () => {
    const gridKeypair = anchor.web3.Keypair.generate()
    const boardKeypair = anchor.web3.Keypair.generate()
    const owner = provider.wallet

    await program.methods
      .createBoard({
        rentalPrice: 1,
        width: 1024,
        height: 1024,
      })
      .accounts({
        board: boardKeypair.publicKey,
        owner: owner.publicKey,
      })
      .signers([boardKeypair])
      .rpc()

    await program.methods
      .createGrid()
      .accounts({
        grid: gridKeypair.publicKey,
        owner: owner.publicKey,
      })
      .signers([gridKeypair])
      .rpc()

    let seeds = ["pad", "0", "0"].map((s) => Buffer.from(s))
    let [padPubKey, _] = anchor.web3.PublicKey.findProgramAddressSync(seeds, program.programId)

    const beastieKeypair = anchor.web3.Keypair.generate()
    await program.methods
      .place(100, 100, 40)
      .accounts({
        grid: gridKeypair.publicKey,
        board: boardKeypair.publicKey,
        owner: owner.publicKey,
        beastie: beastieKeypair.publicKey,
      })
      .remainingAccounts([
        { isSigner: false, isWritable: true, pubkey: padPubKey }
      ])
      .signers([beastieKeypair])
      .rpc()
  })
})
