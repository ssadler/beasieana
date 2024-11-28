//use anchor_lang::prelude::*;
//use crate::state::pad::add_cell_to_pad;
//use crate::types::*;
//
//
//use anchor_lang::solana_program::{
//    program::invoke_signed,
//    pubkey::Pubkey,
//    system_instruction,
//    sysvar::rent::Rent,
//    sysvar::Sysvar,
//};
//
//
//#[derive(Accounts)]
//pub struct PlaceBeastie<'info> {
//    pub beastie: Account<'info, beastie::state::
//    pub payer: Signer<'info>
//}
//
//pub fn place_beastie_on_grid<'info>(ctx: Context<'_, '_, '_, 'info, PlaceBeastie<'info>>, cell_id: u32, pos: CellPos) -> Result<()> {
//
//    // Check in bounds // TODO check upper bounds
//    if pos.x < pos.r || pos.y < pos.r {
//        panic!("OOB");
//    }
//
//    // TODO check min size, max size, min balance
//
//    // TODO: this will overflow, cast to u32
//    let xmin = (pos.x-pos.r as u16) >> 8;
//    let xmax = (pos.x+pos.r as u16) >> 8;
//    let ymin = (pos.y-pos.r as u16) >> 8;
//    let ymax = (pos.y+pos.r as u16) >> 8;
//
//    let mut pda_idx = 0;
//    
//    for xx in xmin..(xmax+1) {
//        let xs = xx.to_string();
//        let xb = xs.as_bytes();
//        for yy in ymin..(ymax+1) {
//            let ys = yy.to_string();
//            let yb = ys.as_bytes();
//
//            let account = ctx.remaining_accounts.get(pda_idx).expect("missing pad");
//            pda_idx += 1;
//
//            let (pda, bump_seed) = Pubkey::find_program_address(&[b"pad", &xb, &yb], ctx.program_id);
//            if account.key != &pda {
//                panic!("wrong pad");
//            }
//
//            if !account.is_writable {
//                panic!("pad account not mutable");
//            }
//
//            if account.data_is_empty() {
//                let size = 10240;
//                let reb = Rent::get()?.minimum_balance(size);
//                let payer = ctx.accounts.payer.to_account_info();
//
//                invoke_signed(
//                    &system_instruction::create_account(
//                        payer.key,
//                        account.key,
//                        reb,
//                        size as u64,
//                        ctx.program_id
//                    ),
//                    &[payer, account.clone()],
//                    &[ &[b"pad", &xb, &yb, &[bump_seed]] ]
//                )?;
//            }
//
//            add_cell_to_pad(account.data.borrow_mut(), cell_id, &pos);
//        }
//    }
//
//    Ok(())
//}
//
