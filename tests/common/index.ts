
import * as anchor from "@coral-xyz/anchor";
import { Grid } from "../../target/types/grid";
import { Beastie } from "../../target/types/beastie";


// Configure the client to use the local cluster.
let provider = anchor.AnchorProvider.env()
anchor.setProvider(provider)

const wallet = provider.wallet as anchor.Wallet

export const gridApp = anchor.workspace.Grid as anchor.Program<Grid>
export const beastieApp = anchor.workspace.Beastie as anchor.Program<Beastie>

let init = false

export async function createBeastie(owner?: anchor.web3.PublicKey) {

  if (!init) {
    await beastieApp.methods.initialize().rpc()
    init = true
  }

  owner ||= wallet.publicKey
  let call = beastieApp.methods.createBeastie(owner)
  await call.rpc()
  let accts = await call.pubkeys()
  return {
    placement: {
      ...await gridApp.account.gridBeastie.fetch(accts.placement),
      address: accts.placement
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
