import * as anchor from "@coral-xyz/anchor";
import * as Token from "@solana/spl-token";
import { Beastie, buildProxy, createBeastie, createBoard, getPadATAs, gridApp } from './common'
import {assert} from "chai";

describe("grid", () => {
  // Configure the client to use the local cluster.
  let provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)
  const owner = provider.wallet as anchor.Wallet



  it("Admin Init", async () => {
    await gridApp.methods.adminInit().rpc()
  })


  it("Init Beastie", async () => {
    let { beastie } = await createBeastie()
    assert.equal(beastie.cellId, 1)
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
    board = await createBoard(mint)
  })

  let placed: Beastie

  function initPlacementCall(beastie: Beastie) {
    return gridApp.methods
      .initPlacement()
      .accountsPartial({
        c: {
          beastie: beastie.address,
          board,
          tokenMint: mint,
        },
        cell: beastie.cell,
        tokenMint: mint,
        beastieAta: Token.getAssociatedTokenAddressSync(mint, beastie.address, true),
      })
  }

  it("Init Placement", async () => {
    let beastie = await createBeastie()
    let call = initPlacementCall(beastie)
    //let payer = (await call.prepare()).pubkeys.c.payer
    await buildProxy(beastie.address, await call.instruction()).rpc()
  })

  it("Place beastie", async () => {
    let beastie = placed = await createBeastie()
    let { pads } = await placeBeastie(beastie, { x: 200, y: 200, r: 200 })

    let p = await provider.connection.getAccountInfo(pads[0].pubkey)
    assert.deepEqual([...p.data.subarray(0, 14)], [1,0,0,0,beastie.beastie.cellId,0,0,0,200,0,200,0,200,0])
  })

  it("Remove beastie", async () => {
    let pos = { x: 200, y: 200, r: 200 }
    let pads = getPadATAs(pos)
    let removeCall = gridApp.methods
      .remove()
      .accountsPartial({
        c: {
          beastie: placed.address,
          board,
          tokenMint: mint
        },
        cell: placed.cell,
        beastieAta: Token.getAssociatedTokenAddressSync(mint, placed.address, true),
      })
      .remainingAccounts(pads)

    let call = buildProxy(placed.address, await removeCall.instruction())
    //try {
      await call.rpc()
    //} catch (e) {
    //  let txid = String(e).split(' ')[3]
    //  let tx = await provider.connection.getTransaction(txid, {
    //    commitment: "confirmed",
    //  })
    //  throw e
    //}
    for (let pad of pads) {
      let p = await provider.connection.getAccountInfo(pad.pubkey)
      assert.deepEqual([...p.data.subarray(0, 14)], [0,0,0,0,0,0,0,0,0,0,0,0,0,0])
    }
  })

  it("Shrink beastie", async () => {
    let beastie = await createBeastie()
    let pos = { x: 100, y: 100, r: 100 }
    let b = await placeBeastie(beastie, pos)

    let beastie2 = await createBeastie()
    let pos2 = { x: 380, y: 100, r: 100 }
    await placeBeastie(beastie2, pos2, { interacts: [b] })
  })
})

