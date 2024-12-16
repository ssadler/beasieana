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
    let { cell } = await createBeastie()
    assert.equal(cell.cellId, 1)
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
      maxRadius: 512,
      linkMaxDistance: 1024,
    }
    let call = gridApp.methods
      .createBoard(new anchor.BN(1), owner.publicKey, mint, config)
      .accountsPartial({
        tokenMint: mint
      })
    board = (await call.pubkeys()).board
    await call.rpc()
  })

  type Beastie = Awaited<ReturnType<typeof createBeastie>>
  type Placement = Awaited<ReturnType<typeof placeBeastie>>
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
        cell: beastie.cell.address,
        tokenMint: mint,
        beastieAta: Token.getAssociatedTokenAddressSync(mint, beastie.address, true),
      })
  }

  async function placeBeastie(beastie: Beastie, pos: Pos, opts: { interacts?: Placement[] }={}) {

    let beastieATA = await Token.getOrCreateAssociatedTokenAccount(provider.connection, owner.payer, mint, beastie.address, true)
    await Token.mintTo(provider.connection, owner.payer, mint, beastieATA.address, mintAuthority, 1000000)


    let pads = getPadATAs(pos, ...(opts.interacts||[]).map((b) => b.pos))
    let toAccountInfo = (f: (b: Placement) => anchor.web3.PublicKey) => (opts.interacts||[]).map((b) => ({ isWritable: false, isSigner: false, pubkey: f(b) }))
    let remaining = [
        ...pads,
        ...toAccountInfo((b) => b.beastie.address),
        ...toAccountInfo((b) => b.beastieATA.address),
        ...toAccountInfo((b) => b.beastie.cell.address),
      ]

    let placeCall = gridApp.methods
      .place(pos)
      .accountsPartial({
        c: {
          beastie: beastie.address,
          board,
          tokenMint: mint
        },
        cell: beastie.cell.address,
        beastieAta: Token.getAssociatedTokenAddressSync(mint, beastie.address, true),
      })
      .remainingAccounts(remaining)

    let initPlacement = buildProxy(beastie.address, await initPlacementCall(beastie).instruction())
    let r = await buildProxy(beastie.address, await placeCall.instruction())
      .preInstructions([await initPlacement.instruction()], true)
      .rpc()

    //await new Promise((r) => setTimeout(r, 100))
    //let tx = await provider.connection.getTransaction(r, { commitment: 'confirmed' })
    //console.log(`placeBeastie: ${tx.meta.computeUnitsConsumed} CU`)

    return {
      beastieATA,
      beastie,
      pos,
      pads
    }
  }

  it("Init Placement", async () => {
    let beastie = await createBeastie()
    let call = initPlacementCall(beastie)
    await buildProxy(beastie.address, await call.instruction()).rpc()
  })

  it("Place beastie", async () => {
    let beastie = placed = await createBeastie()
    let { pads } = await placeBeastie(beastie, { x: 200, y: 200, r: 200 })

    let p = await provider.connection.getAccountInfo(pads[0].pubkey)
    assert.deepEqual([...p.data.subarray(0, 14)], [1,0,0,0,beastie.cell.cellId,0,0,0,200,0,200,0,200,0])
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
        cell: placed.cell.address,
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


type Pos = { x: number, y: number, r: number }


function getPadATAs(...positions: Pos[]) {
  const out: { [k: string]: anchor.web3.AccountMeta } = {}

  for (let pos of positions) {
    let xmin = (pos.x - pos.r) >> 9;
    let xmax = (pos.x + pos.r) >> 9;
    let ymin = (pos.y - pos.r) >> 9;
    let ymax = (pos.y + pos.r) >> 9;

    for (let xx=xmin; xx<=xmax; xx++) {
      for (let yy=ymin; yy<=ymax; yy++) {
        //if (!circleOverlapsPad(pos.x, pos.y, pos.r, xx, yy, 512, 512)) continue;
        let seeds = [
          Buffer.from("pad"),
          Buffer.from(""),
          new anchor.BN(xx).toBuffer('le', 2),
          new anchor.BN(yy).toBuffer('le', 2)
        ]
        let [pubkey, _] = anchor.web3.PublicKey.findProgramAddressSync(seeds, gridApp.programId)
        out[pubkey.toString()] = { isSigner: false, isWritable: true, pubkey }
      }
    }
  }
  return Object.values(out)
}


function circleOverlapsPad(cx: number, cy: number, cr: number, x: number, y: number, w: number, h: number): boolean {
    // Multiply sizes by 2 for integer division
    const r32 = cr * 2;
    const w32 = w;
    const h32 = h;

    // px and py are the center points of the pad
    const px = x * 2 + w32;
    const py = y * 2 + h32;

    // dx and dy are the distances from the center points
    const dx = Math.abs(cx * 2 - px);
    const dy = Math.abs(cy * 2 - py);

    if (dx >= w32 + r32 || dy >= h32 + r32) {
        return false;
    } else if (dx <= w32 || dy <= h32) {
        return true;
    } else {
        return Math.pow(dx - w32, 2) + Math.pow(dy - h32, 2) <= Math.pow(r32, 2);
    }
}
