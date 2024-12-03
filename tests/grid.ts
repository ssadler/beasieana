import * as anchor from "@coral-xyz/anchor";
import * as Token from "@solana/spl-token";
import { buildProxy, createBeastie, gridApp } from './common'
import {assert} from "chai";


describe("grid", () => {
  // Configure the client to use the local cluster.
  let provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)
  const owner = provider.wallet as anchor.Wallet



  it("Admin Init", async () => {
    await new Promise((r) => setTimeout(r, 2000))
    await gridApp.methods.adminInit().rpc()
  })


  it("Init Beastie", async () => {
    let seed = Math.floor(Math.random()*1000000000)

    let { gridBeastie } = await createBeastie(seed)

    assert.equal(gridBeastie.seed.toNumber(), seed)
    assert.equal(gridBeastie.cellId, 1)
  })


  const mintAuthority = anchor.web3.Keypair.generate()
  let mint: anchor.web3.PublicKey

  it("Whitelist token", async () => {
    mint = await Token.createMint(
      provider.connection,
      owner.payer,
      mintAuthority.publicKey,
      null,
      9
    )
    await gridApp.methods.adminWhitelistToken(mint).rpc()
  })

  let board: anchor.web3.PublicKey

  it("Create Board", async () => {
    let config = {
      rate: new anchor.BN(1),
      width: 1024,
      height: 1024,
      addCellMinValue: new anchor.BN(100000),
      minRadius: 64,
      maxRadius: 512
    }
    let call = gridApp.methods
      .createBoard(new anchor.BN(1), owner.publicKey, mint, config)
      .accountsPartial({
        tokenMint: mint
      })
    board = (await call.pubkeys()).board
    await call.rpc()
  })


  it("Place beastie", async () => {
    let beastie = await createBeastie(3)

    let beastieATA = await Token.getOrCreateAssociatedTokenAccount(provider.connection, owner.payer, mint, beastie.address, true)
    await Token.mintTo(provider.connection, owner.payer, mint, beastieATA.address, mintAuthority, 1000000)

    let pos = { x: 200, y: 200, r: 200 }
    let placeCall = gridApp.methods
      .place(pos)
      .accountsPartial({
        assetBeastie: beastie.address,
        gridBeastie: beastie.gridBeastie.address,
        board,
        tokenMint: mint
      })
      .remainingAccounts(
        getPadATAs(board, pos).map((pubkey) => ({ isSigner: false, isWritable: true, pubkey }))
      )

    await buildProxy(beastie.address, await placeCall.instruction()).rpc()
  })
})



function getPadATAs(board: anchor.web3.PublicKey, pos: { x: number, y: number, r: number }) {
  let xmin = (pos.x - pos.r) >> 9;
  let xmax = (pos.x + pos.r) >> 9;
  let ymin = (pos.y - pos.r) >> 9;
  let ymax = (pos.y + pos.r) >> 9;

  let out = []
  
  for (let xx=xmin; xx<=xmax; xx++) {
    for (let yy=ymin; yy<=ymax; yy++) {
      let seeds = [
        Buffer.from("pad"),
        board.toBuffer(),
        new anchor.BN(xx).toBuffer('le', 2),
        new anchor.BN(yy).toBuffer('le', 2)
      ]
      let [padPubKey, _] = anchor.web3.PublicKey.findProgramAddressSync(seeds, gridApp.programId)
      out.push(padPubKey)
    }
  }
  console.log("pad ATAs: ", out.length)
  return out
}
