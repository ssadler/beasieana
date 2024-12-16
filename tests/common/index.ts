
import * as anchor from "@coral-xyz/anchor";
import { Grid } from "../../target/types/grid";
import { Beastie } from "../../target/types/beastie";


// Configure the client to use the local cluster.
let provider = anchor.AnchorProvider.env()
anchor.setProvider(provider)

const wallet = provider.wallet as anchor.Wallet

export const gridApp = anchor.workspace.Grid as anchor.Program<Grid>
export const beastieApp = anchor.workspace.Beastie as anchor.Program<Beastie>


let next_cell_id = 1
export async function createBeastie(owner?: anchor.web3.PublicKey, cell_id?: number) {

  cell_id ??= next_cell_id++

  owner ||= wallet.publicKey
  let call = beastieApp.methods.createBeastie(cell_id, owner)
  await call.rpc()
  let accts = await call.pubkeys()
  return {
    cell: {
      ...await gridApp.account.cell.fetch(accts.cell),
      address: accts.cell
    },
    beastie: await beastieApp.account.beastie.fetch(accts.beastie),
    address: accts.beastie
  }
}


export function buildProxy(beastie: anchor.web3.PublicKey, call: anchor.web3.TransactionInstruction) {
  return beastieApp.methods
    .proxy(call.data)
    .accountsPartial({ beastie })
    .remainingAccounts([
      { pubkey: call.programId, isSigner: false, isWritable: false },
      ...call.keys.map((k) => k.pubkey.equals(beastie) ? {...k, isSigner: false} : k)
    ])
}
