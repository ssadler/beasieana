import * as anchor from "@coral-xyz/anchor";
import { beastieAddress, beastieApp, buildProxyMulti, cellAddress, createBeastie, globalPlacementContextAccounts, gridApp, placeBeastie } from './common'
import {assert} from "chai";


let y = 900

describe("proxy", () => {
  // Configure the client to use the local cluster.
  let provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)
  //const owner = provider.wallet as anchor.Wallet


  it("Simple internal calls", async () => {
    let beastie = await createBeastie()

    let noop = gridApp.methods.noop().accountsPartial({ beastie: beastie.address })
    let txid = await buildProxyMulti(beastie.address, [noop, noop]).rpc()

    await new Promise((r) => setTimeout(r, 100))
    let tx = await provider.connection.getTransaction(txid, {
      commitment: "confirmed",
    })

    assert.deepEqual(
      tx.meta.logMessages.filter((m) => m.indexOf("consumed") == -1),
      [
        `Program ${beastieApp.programId} invoke [1]`,
        `Program log: Instruction: Proxy`,
        `Program ${gridApp.programId} invoke [2]`,
        `Program log: Instruction: Noop`,
        `Program ${gridApp.programId} success`,
        `Program ${gridApp.programId} invoke [2]`,
        `Program log: Instruction: Noop`,
        `Program ${gridApp.programId} success`,
        `Program ${beastieApp.programId} success`,
      ]
    )
  })

  it("Fail when wrong preflight program", async () => {
    let beastie = await createBeastie()
    let call = beastieApp.methods.noop()
    try {
      await buildProxyMulti(beastie.address, [call]).rpc()
      assert.fail("expected to fail with wrong preflight")
    } catch (e) {
      assert.isTrue(e.transactionLogs[2].indexOf("Preflight program must be grid") >= 0)
    }
  })

  it("Fail when preflight first account not beastie", async () => {
    let beastie = await createBeastie()
    let instr = gridApp.methods.noop()
    // need to provide a noop so that it's identified as ext
    let noop = beastieApp.methods.noop()
    try {
      await buildProxyMulti(beastie.address, [instr, noop]).rpc()
      assert.fail("expected to fail with wrong preflight")
    } catch (e) {
      assert.isTrue(e.logs[2].indexOf("Preflight first account must be beastie") >= 0)
    }
  })

  it("External call fails when wrong preflight", async () => {
    let beastie = await createBeastie()
    let instr = gridApp.methods.noop()
      .accountsPartial({ beastie: beastie.address })
    // need to provide a noop so that it's identified as ext
    let noop = beastieApp.methods.noop()
    try {
      await buildProxyMulti(beastie.address, [instr, noop]).rpc()
      assert.fail("expected to fail with wrong preflight")
    } catch (e) {
      assert.isTrue(e.logs[2].indexOf("Preflight call unexpected instruction") >= 0)
    }
  })

  it("External call succeeds when cell not active", async () => {
    let beastie = await createBeastie()

    let instr = gridApp.methods
      .beastieIsActive()
      .accountsPartial({ beastie: beastie.address, cell: beastie.cell })
    // need to provide a noop so that it's identified as ext
    let noop = beastieApp.methods.noop()
    let txid = await buildProxyMulti(beastie.address, [instr, noop]).rpc()

    await new Promise((r) => setTimeout(r, 100))
    let tx = await provider.connection.getTransaction(txid, {
      commitment: "confirmed",
    })

    assert.deepEqual(
      tx.meta.logMessages.filter((m) => m.indexOf("consumed") == -1),
      [
        `Program ${beastieApp.programId} invoke [1]`,
        `Program log: Instruction: Proxy`,
        `Program ${gridApp.programId} invoke [2]`,
        `Program log: Instruction: BeastieIsActive`,
        `Program return: ${gridApp.programId} AA==`,
        `Program ${gridApp.programId} success`,
        `Program ${beastieApp.programId} invoke [2]`,
        `Program log: Instruction: Noop`,
        `Program ${beastieApp.programId} success`,
        `Program ${beastieApp.programId} success`,
      ]
    )
  })

  it("Fail when preflight not bill and is active", async () => {
    let beastie = await createBeastie()
    await placeBeastie(beastie, { x: 100, y, r: 90 })

    let instr = gridApp.methods.beastieIsActive()
      .accountsPartial({ beastie: beastie.address, cell: beastie.cell })
    // need to provide a noop so that it's identified as ext
    let noop = beastieApp.methods.noop()

    try {
      await buildProxyMulti(beastie.address, [instr, noop]).rpc()
      assert.fail("expected to fail with wrong preflight")
    } catch (e) {
      assert.isTrue(e.logs[7].indexOf("Preflight: beastie is active") >= 0)
    }
  })

  it("Succeed when active and preflight is correct", async () => {
    let beastie = await createBeastie()
    await placeBeastie(beastie, { x: 300, y, r: 90 })

    let instr = gridApp.methods.billMe()
      .accountsPartial(globalPlacementContextAccounts(beastie))
    // need to provide a noop so that it's identified as ext
    let noop = beastieApp.methods.noop()

    let txid = await buildProxyMulti(beastie.address, [instr, noop]).rpc()

    await new Promise((r) => setTimeout(r, 100))
    let tx = await provider.connection.getTransaction(txid, {
      commitment: "confirmed",
    })
    assert.deepEqual(
      tx.meta.logMessages.filter((m) => m.indexOf("consumed") == -1),
      [
        `Program ${beastieApp.programId} invoke [1]`,
        `Program log: Instruction: Proxy`,
        `Program ${gridApp.programId} invoke [2]`,
        `Program log: Instruction: BillMe`,
        `Program return: ${gridApp.programId} AQ==`,
        `Program ${gridApp.programId} success`,
        `Program ${beastieApp.programId} invoke [2]`,
        `Program log: Instruction: Noop`,
        `Program ${beastieApp.programId} success`,
        `Program ${gridApp.programId} invoke [2]`,
        `Program log: Instruction: CheckMe`,
        `Program ${gridApp.programId} success`,
        `Program ${beastieApp.programId} success`,
      ]
    )
  })

  it("Fail proxy without preflight if notice is pending", async () => {
    let beastie = await createBeastie()
    await beastieApp.methods.giveNotice()
      .accountsPartial({ beastie: beastie.address })
      .rpc()

    let noop = beastieApp.methods.noop()
    try {
      await buildProxyMulti(beastie.address, [noop]).rpc()
      assert.fail("expected to fail with wrong preflight")
    } catch (e) {
      assert.isTrue(e.logs[2].indexOf("\nPreflight program must be grid") >= 0)
    }
  })

  it("Succeed proxy without preflight if notice is fulfilled", async () => {
    let beastie = await createBeastie()
    await placeBeastie(beastie, { x: 500, y, r: 90 })
    await beastieApp.methods
      .systemOverride({ 'setNoticeFulfilled': {} })
      .accountsPartial({ beastie: beastie.address })
      .rpc()

    let i = anchor.web3.SystemProgram.transfer({
      fromPubkey: provider.wallet.publicKey,
      toPubkey: beastie.address,
      lamports: 1
    })

    await buildProxyMulti(beastie.address, [i]).rpc()
  })
})

