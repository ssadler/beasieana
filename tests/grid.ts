import * as anchor from "@coral-xyz/anchor";
import { Grid } from "../target/types/grid";
import { Beastie } from "../target/types/beastie";
import { createBeastie } from './common'

import * as Token from "@solana/spl-token";
import {assert} from "chai";

describe("grid", () => {
  // Configure the client to use the local cluster.
  let provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const wallet = provider.wallet as anchor.Wallet

  //const progGrid = anchor.workspace.Grid as anchor.Program<Grid>
  const progBeastie = anchor.workspace.Beastie as anchor.Program<Beastie>


  it("Init Beastie", async () => {
    let seed = Math.floor(Math.random()*1000000000)

    let { gridBeastie } = await createBeastie(seed)

    assert.equal(gridBeastie.seed.toNumber(), seed)
    assert.equal(gridBeastie.cellId, 1)
  })


  it("Sends Token", async () => {

    const b = await createBeastie()

    const mintAuthority = anchor.web3.Keypair.generate()
    const mint = await Token.createMint(
      provider.connection,
      wallet.payer,
      mintAuthority.publicKey,
      null,
      9
    )
    let beastieATA = await Token.getOrCreateAssociatedTokenAccount(provider.connection, wallet.payer, mint, b.address, true)
    await Token.mintTo(provider.connection, wallet.payer, mint, beastieATA.address, mintAuthority, 100)

    let walletATA = await Token.getOrCreateAssociatedTokenAccount(provider.connection, wallet.payer, mint, wallet.publicKey)

    let call = progBeastie.methods.sendToken(new anchor.BN(100))
      .accountsPartial({
        beastieAta: beastieATA.address,
        destAta: walletATA.address,
        beastie: b.address
      })

    //console.log(await call.pubkeys())

    await call.rpc()
  })


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

  //it("Adds cell to pad", async () => {
  //  const gridKeypair = anchor.web3.Keypair.generate()
  //  const boardKeypair = anchor.web3.Keypair.generate()
  //  const mintAuthority = anchor.web3.Keypair.generate()
  //  const owner = provider.wallet as anchor.Wallet

  //  const mint = await Token.createMint(
  //    provider.connection,
  //    owner.payer,
  //    mintAuthority.publicKey,
  //    null,
  //    9
  //  )




  //  await program.methods
  //    .createBoard({
  //      rentalPrice: 1,
  //      width: 1024,
  //      height: 1024,
  //    })
  //    .accounts({
  //      board: boardKeypair.publicKey,
  //      owner: owner.publicKey,
  //    })
  //    .signers([boardKeypair])
  //    .rpc()

  //  await program.methods
  //    .createGrid()
  //    .accounts({
  //      grid: gridKeypair.publicKey,
  //      owner: owner.publicKey,
  //    })
  //    .signers([gridKeypair])
  //    .rpc()

  //  let seeds = ["pad", "0", "0"].map((s) => Buffer.from(s))
  //  let [padPubKey, _] = anchor.web3.PublicKey.findProgramAddressSync(seeds, program.programId)

  //  const beastieKeypair = anchor.web3.Keypair.generate()
  //  await program.methods
  //    .place(100, 100, 40)
  //    .accounts({
  //      grid: gridKeypair.publicKey,
  //      board: boardKeypair.publicKey,
  //      owner: owner.publicKey,
  //      beastie: beastieKeypair.publicKey,
  //    })
  //    .remainingAccounts([
  //      { isSigner: false, isWritable: true, pubkey: padPubKey }
  //    ])
  //    .signers([beastieKeypair])
  //    .rpc()

  //})
})
