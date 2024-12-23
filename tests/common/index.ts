
import * as anchor from "@coral-xyz/anchor";
import { Grid } from "../../target/types/grid";
import { Beastie as BeastieProgram } from "../../target/types/beastie";
import * as Token from "@solana/spl-token";
import {MethodsBuilder} from "@coral-xyz/anchor/dist/cjs/program/namespace/methods";


// Configure the client to use the local cluster.
let provider = anchor.AnchorProvider.env()
anchor.setProvider(provider)
const owner = provider.wallet as anchor.Wallet

const wallet = provider.wallet as anchor.Wallet

export const gridApp = anchor.workspace.Grid as anchor.Program<Grid>
export const beastieApp = anchor.workspace.Beastie as anchor.Program<BeastieProgram>

export function beastieAddress(cell_id: number) {
  let seeds = [Buffer.from("beastie"), new anchor.BN(cell_id).toBuffer('le', 4)]
  return anchor.web3.PublicKey.findProgramAddressSync(seeds, beastieApp.programId)
}

export function cellAddress(cell_id: number) {
  let seeds = [Buffer.from("cell"), new anchor.BN(cell_id).toBuffer('le', 4)]
  return anchor.web3.PublicKey.findProgramAddressSync(seeds, gridApp.programId)
}

let next_cell_id = 1
export type Beastie = Awaited<ReturnType<typeof createBeastie>>
export async function createBeastie(owner?: anchor.web3.PublicKey, cell_id?: number) {
  cell_id ??= next_cell_id++
  owner ||= wallet.publicKey

  let beastie = beastieAddress(cell_id)[0]
  let cell = cellAddress(cell_id)[0]
  let call = beastieApp.methods.createBeastie(cell_id, owner).accountsPartial({
    beastie,
    cell
  })
  return {
    cell_id,
    txid: await call.rpc(),
    cell,
    beastie: await beastieApp.account.beastie.fetch(beastie),
    address: beastie
  }
}

let next_board_id = 1
export async function createBoard(mint: anchor.web3.PublicKey, owner?: anchor.web3.PublicKey, id?: number) {
  id ??= next_board_id++
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
    .createBoard(new anchor.BN(1), owner, mint, config)
    .accountsPartial({
      tokenMint: mint
    })
  let board = (await call.pubkeys()).board
  await call.rpc()
  return board
}

type CallBuilder = anchor.web3.TransactionInstruction | MethodsBuilder<any, any>
export function buildProxyMulti(
  beastie: anchor.web3.PublicKey,
  proxyCalls: CallBuilder[],
) {

  let allAccounts: anchor.web3.AccountMeta[] = [
    { isSigner: false, isWritable: false, pubkey: beastie }
  ]

  function getAccountIdx(account: anchor.web3.AccountMeta) {
    let idx = allAccounts.findIndex((a) => String(a.pubkey) == String(account.pubkey))
    if (idx >= 0) {
      allAccounts[idx].isSigner ||= idx > 0 && account.isSigner
      allAccounts[idx].isWritable ||= account.isWritable
      return idx
    }
    allAccounts.push(account)
    return allAccounts.length - 1
  }


  let runCalls = async () => {
    let instructions = await Promise.all(
      proxyCalls.map(async (p) => (
        p instanceof anchor.web3.TransactionInstruction ? p : await p.instruction()
      ))
    )

    return instructions.map((instruction) => {
      let accounts = instruction.keys.map((a) => 
        getAccountIdx(a) | (a.isWritable ? 64 : 0) | (a.isSigner ? 128 : 0)
      )
      return {
        accounts: Buffer.from(accounts),
        data: instruction.data,
        programIdx: getAccountIdx({ pubkey: instruction.programId, isSigner: false, isWritable: false })
      }
    })
  }

  let pcalls = runCalls()

  return {
    rpc: async () => {
      let calls = await pcalls
      return beastieApp.methods.proxy(await pcalls).remainingAccounts(allAccounts).rpc()
    },
    prepare: async () => {
      return beastieApp.methods.proxy(await pcalls).remainingAccounts(allAccounts).prepare()
    }
  }
}

export function placementContextAccounts(
  beastie: Beastie,
  board: anchor.web3.PublicKey,
  mint: anchor.web3.PublicKey
) {
  return {
    c: {
      beastie: beastie.address,
      board,
      tokenMint: mint
    },
    cell: beastie.cell,
    beastieAta: Token.getAssociatedTokenAddressSync(mint, beastie.address, true),
  }
}

export function globalPlacementContextAccounts(beastie: Beastie) {
  return placementContextAccounts(
    beastie,
    globalPlacementContext.board,
    globalPlacementContext.mint
  )
}


export function buildProxy(beastie: anchor.web3.PublicKey, call: anchor.web3.TransactionInstruction) {
  return buildProxyMulti(beastie, [call])
}


export type PlacementContext = {
  mint: anchor.web3.PublicKey,
  board: anchor.web3.PublicKey,
  mintAuthority: anchor.web3.Keypair,
}

const globalPlacementContext: PlacementContext = {
  mintAuthority: anchor.web3.Keypair.generate()
} as any

export type Placement = Awaited<ReturnType<typeof placeBeastie>>
export async function placeBeastie(beastie: Beastie, pos: Pos, { interacts, ctx }: { interacts?: Placement[], ctx?: PlacementContext }={}) {

  if (!ctx) {
    ctx = globalPlacementContext
    if (!ctx.board) {
      await gridApp.methods.adminInit().rpc()
      ctx.mint = await Token.createMint(
        provider.connection,
        owner.payer,
        ctx.mintAuthority.publicKey,
        null,
        9
      )
      await gridApp.methods.adminWhitelistToken(ctx.mint).rpc()
      ctx.board = await createBoard(ctx.mint)
    }
  }

  let { mint, board, mintAuthority } = ctx
  let beastieATA = await Token.getOrCreateAssociatedTokenAccount(provider.connection, owner.payer, mint, beastie.address, true)
  await Token.mintTo(provider.connection, owner.payer, mint, beastieATA.address, mintAuthority, 1000000)


  let pads = getPadATAs(pos, ...(interacts||[]).map((b) => b.pos))
  let toAccountInfo = (f: (b: Placement) => anchor.web3.PublicKey) => (interacts||[]).map((b) => ({ isWritable: false, isSigner: false, pubkey: f(b) }))
  let remaining = [
      ...pads,
      ...toAccountInfo((b) => b.beastie.address),
      ...toAccountInfo((b) => b.beastieATA.address),
      ...toAccountInfo((b) => b.beastie.cell),
    ]

  let placeCall = gridApp.methods
    .place(pos)
    .accountsPartial({
      c: {
        beastie: beastie.address,
        board,
        tokenMint: mint
      },
      cell: beastie.cell,
      beastieAta: Token.getAssociatedTokenAddressSync(mint, beastie.address, true),
    })
    .remainingAccounts(remaining)

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

  let initPlacement = buildProxyMulti(beastie.address, [
    initPlacementCall(beastie),
    placeCall
  ])

  await initPlacement.rpc()

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



export type Pos = { x: number, y: number, r: number }

export function getPadATAs(...positions: Pos[]) {
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


export function circleOverlapsPad(cx: number, cy: number, cr: number, x: number, y: number, w: number, h: number): boolean {
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
