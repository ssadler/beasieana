import * as anchor from "@coral-xyz/anchor"
import { beastieApp, buildProxy, createBeastie, gridApp } from './common'
import {assert, expect} from "chai";

import * as Token from "@solana/spl-token";

describe("beastie", () => {
  let provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)
  const owner = provider.wallet as anchor.Wallet

  let beastie

  it("Create Beastie", async () => {

    let r = beastie = await createBeastie(owner.publicKey)
    assert.equal(r.beastie.cellId, 1)
    assert.equal(r.beastie.owner.toString(), owner.publicKey.toString())

    r = await createBeastie(owner.publicKey)
    assert.equal(r.beastie.cellId, 2)
  })

  const wallet = provider.wallet as anchor.Wallet

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
    await Token.mintTo(provider.connection, wallet.payer, mint, beastieATA.address, mintAuthority, 102)

    let walletATA = await Token.getOrCreateAssociatedTokenAccount(provider.connection, wallet.payer, mint, wallet.publicKey)

    let transfer = Token.createTransferInstruction(beastieATA.address, walletATA.address, b.address, 101)
    await buildProxy(b.address, transfer).rpc()

    walletATA = await Token.getAccount(provider.connection, walletATA.address)
    expect(walletATA.amount.toString()).to.equal("101")

    beastieATA = await Token.getAccount(provider.connection, beastieATA.address)
    expect(beastieATA.amount.toString()).to.equal("1")
  })

})
