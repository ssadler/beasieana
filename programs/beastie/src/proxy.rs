use anchor_lang:: {
    prelude::*,
    solana_program::{instruction::Instruction, program}, InstructionData
};
use beastie_common::*;


#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ProxyCall {
    data: Vec<u8>,
    program_idx: u8,
    accounts: Vec<u8>
}


#[derive(Accounts)]
pub struct MultiProxyContext<'info> {
    pub owner: Signer<'info>,
}


pub fn multi_proxy<'info>(
    ctx: Context<'_, '_, '_, 'info, MultiProxyContext<'info>>,
    calls: Vec<ProxyCall>
) -> Result<()> {

    /*
     * Get and authenticate beastie
     */
    let beastie = Beastie::try_deserialize(&mut ctx.remaining_accounts[0].data.borrow().as_ref())?;
    assert!(&beastie.owner == ctx.accounts.owner.key, "unauthorized owner");
    let seeds = &mut [BEASTIE_KEY, byte_ref!(beastie.cell_id, 4), &[]];
    let (addr, bump) = Pubkey::find_program_address(&seeds[0..2], ctx.program_id);
    assert!(addr == *ctx.remaining_accounts[0].key, "unauthorized beastie");
    let bump = &[bump];
    seeds[2] = bump;

    let get_program_id = |c: &ProxyCall| ctx.remaining_accounts[c.program_idx as usize].key;

    let run_call = |pc: ProxyCall| -> Result<()> {
        let program_id = *get_program_id(&pc);
        let accounts = pc.accounts.into_iter().map(|spec| {
            let idx = spec as usize & 63;
            AccountMeta {
                is_writable: spec & 64 > 0,
                is_signer: idx == 0 || spec & 128 > 0,
                pubkey: *ctx.remaining_accounts[idx].key
            }
        }).collect();

        program::invoke_signed_unchecked(
            &Instruction::new_with_bytes(program_id, &pc.data, accounts),
            &ctx.remaining_accounts,
            &[seeds]
        )
        .map_err(Error::from)
    };

    let run_preflight = |mut p: ProxyCall| -> Result<Option<ProxyCall>> {
        assert!(get_program_id(&p) == &GRID_PROGRAM_ID, "Preflight program must be grid");
        assert!(p.accounts[0] & 63 == 0, "Preflight first account must be beastie");
        if p.data == (grid::instruction::BillMe {}).data() {
            run_call(p.clone())?;
            let (_, r) = program::get_return_data().expect("Invalid response from grid::billme");
            assert!(r.len() == 1, "unexpected grid::billme return data");
            if r[0] != 0 {
                p.data = grid::instruction::CheckMe {}.data();
                return Ok(Some(p));
            }
        } else if p.data == (grid::instruction::VerifyNotActive {}).data() {
            run_call(p)?;
        } else {
            panic!("Preflight call not BillMe or VerifyNotActive");
        }
        Ok(None)
    };

    /*
     * Check if there are any calls that aren't grid
     */
    let has_ext = calls.iter().any(|c| get_program_id(c) != &GRID_PROGRAM_ID);
    let notice = beastie.notice_state()?;
    assert!(notice != NoticeState::Pending, "cant proxy; notice pending");
    let do_checks = has_ext && notice != NoticeState::Fulfilled;
    let mut iter = calls.into_iter();

    /*
     * Preflight billing (so that if there are any withdrawls, billing can happen first)
     */
    let postflight = if do_checks { run_preflight(iter.next().unwrap())? } else { None };

    /*
     * Run each call
     */
    for pc in iter {
        run_call(pc)?;
    }

    /*
     * Finally check that beastie is still valid
     * (board may determine that some conditions are no longer met)
     */
    if let Some(check) = postflight {
        run_call(check)?;
    }

    Ok(())
}

