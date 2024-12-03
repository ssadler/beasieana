use anchor_lang::{
    prelude::*,
    solana_program::{
        program::invoke_signed,
        pubkey::Pubkey,
        system_instruction,
        sysvar::rent::Rent,
        sysvar::Sysvar,
    }
};
use crate::state::pad;
use crate::types::*;
use crate::utils::*;


const RAD_MAX: u16 = 1023;



pub fn place_beastie_on_grid<'a, 'info>(
    program_id: &'a Pubkey,
    remaining_accounts: &'a [AccountInfo<'info>],
    payer: AccountInfo<'info>,
    cell_id: u32,
    space_key: Pubkey,
    pos: CellPos
) -> Result<()> {

    if pos.r > RAD_MAX {
        panic!("RAD_MAX is 1023");
    }
    pos.check_bounded();

    for (pda_idx, (xx, yy)) in pos.pads(9).iter().enumerate() {

        let pad = remaining_accounts.get(pda_idx).expect("missing pad");

        let seeds = [b"pad".as_ref(), space_key.as_ref(), byte_ref!(xx, 2), byte_ref!(yy, 2)];
        let (pda, bump_seed) = Pubkey::find_program_address(&seeds, program_id);

        if pad.key != &pda { panic!("wrong pad"); }
        if !pad.is_writable { panic!("pad account not writeable"); }

        if pad.data_is_empty() {
            let size = 10240;
            let reb = Rent::get()?.minimum_balance(size);

            invoke_signed(
                &system_instruction::create_account(
                    payer.key,
                    pad.key,
                    reb,
                    size as u64,
                    program_id
                ),
                &[payer.clone(), pad.clone()],
                &[ &[b"pad", space_key.as_ref(), byte_ref!(xx, 2), byte_ref!(yy, 2), &[bump_seed]] ]
            )?;
        }

        pad::add_cell_to_pad(pad.data.borrow_mut(), cell_id, &pos);
    }

    Ok(())
}





pub fn remove_beastie_from_grid<'a, 'info>(
    program_id: &'a Pubkey,
    remaining_accounts: &'a [AccountInfo<'info>],
    cell_id: u32,
    space_key: Pubkey,
    pos: CellPos
) -> Result<()> {

    for (pda_idx, (xx, yy)) in pos.pads(9).iter().enumerate() {

        let pad = remaining_accounts.get(pda_idx).expect("missing pad");

        let seeds = [b"pad".as_ref(), space_key.as_ref(), byte_ref!(xx, 2), byte_ref!(yy, 2)];
        let (pda, _) = Pubkey::find_program_address(&seeds, program_id);

        if pad.key != &pda { panic!("wrong pad"); }
        if !pad.is_writable { panic!("pad account not writeable"); }

        if pad.data_is_empty() {
            panic!("pad is not initialized?");
        }

        pad::remove_cell_from_pad(pad.data.borrow_mut(), cell_id);
    }

    Ok(())
}

