
import * as anchor from "@coral-xyz/anchor";
import { Grid } from "../../target/types/grid";
import { Beastie } from "../../target/types/beastie";

import * as Token from "@solana/spl-token";
import {assert} from "chai";

// Configure the client to use the local cluster.
let provider = anchor.AnchorProvider.env()
anchor.setProvider(provider)

const wallet = provider.wallet as anchor.Wallet

const grid = anchor.workspace.Grid as anchor.Program<Grid>
const beastie = anchor.workspace.Beastie as anchor.Program<Beastie>

export async function createBeastie(seed?: number, owner?: anchor.web3.PublicKey) {
  seed ||= Math.floor(Math.random()*1000000000)
  owner ||= wallet.publicKey
  let call = beastie.methods.createBeastie(new anchor.BN(seed), owner)
  await call.rpc()
  let accts = await call.pubkeys()
  return {
    gridBeastie: await grid.account.gridBeastie.fetch(accts.gridBeastie),
    beastie: await beastie.account.beastie.fetch(accts.beastie),
    address: accts.beastie
  }
}
