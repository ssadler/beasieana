import * as anchor from "@coral-xyz/anchor"
import { createBeastie } from './common'
import {assert} from "chai";

describe("beastie", () => {
  let provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  it("Create Beastie", async () => {

    const owner = provider.wallet as anchor.Wallet
    let r = await createBeastie(10, owner.publicKey)

    assert.equal(r.seed.toNumber(), 10)
    assert.equal(r.owner.toString(), owner.publicKey.toString())

    //let instruction = program.methods.createBeastie(new anchor.BN(1), owner.publicKey)

    //let accts = await instruction.pubkeys()
    //await instruction.rpc()
    //let r = await program.account.beastie.fetch(accts['beastie'])

    //assert.equal(r.seed.toNumber(), 1)
    //assert.equal(r.owner.toString(), owner.publicKey.toString())

    //try {
    //  let rand = anchor.web3.Keypair.generate()
    //  await program.methods.forward()
    //    .accountsPartial({
    //      beastie: accts['beastie'],
    //      owner: rand.publicKey
    //    })
    //    .signers([rand])
    //    .rpc()
    //  assert.fail("forward with wrong owner didnt fail")
    //} catch {}
  })
})
