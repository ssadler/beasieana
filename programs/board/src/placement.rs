use anchor_lang::prelude::*;
use crate::state::add_cell_to_pad;
use crate::state::board::*;
use crate::types::*;
use crate::state::grid::*;
use crate::state::beastie::*;

use anchor_lang::solana_program::{
    account_info::AccountInfo,
    program::invoke_signed,
    pubkey::Pubkey,
    system_instruction,
    sysvar::rent::Rent,
    sysvar::Sysvar,
};



#[derive(Accounts)]
pub struct PlaceBeastie<'info> {
    #[account()]
    pub grid: Account<'info, Grid>,
    #[account()]
    pub board: Account<'info, Board>,
    #[account(init, payer = owner, space = 4096)]
    pub beastie: Account<'info, Beastie>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn place_beastie<'info>(ctx: Context<'_, '_, '_, 'info, PlaceBeastie<'info>>, pos: CellPos) -> Result<()> {

    // check init beastie
    if ctx.accounts.beastie.cell_id == 0 {
        ctx.accounts.beastie.cell_id = ctx.accounts.grid.next_cell_id();
    }

    // Check in bounds // TODO check upper bounds
    if pos.x < pos.r || pos.y < pos.r {
        panic!("OOB");
    }

    // TODO check min size, max size, min balance

    // TODO: this will overflow, cast to u32
    let xmin = (pos.x-pos.r as u16) >> 8;
    let xmax = (pos.x+pos.r as u16) >> 8;
    let ymin = (pos.y-pos.r as u16) >> 8;
    let ymax = (pos.y+pos.r as u16) >> 8;

    let mut pda_idx = 0;
    
    for xx in xmin..(xmax+1) {
        let xs = xx.to_string();
        let xb = xs.as_bytes();
        for yy in ymin..(ymax+1) {
            let ys = yy.to_string();
            let yb = ys.as_bytes();

            let account = ctx.remaining_accounts.get(pda_idx).expect("missing pad");
            pda_idx += 1;

            let (pda, bump_seed) = Pubkey::find_program_address(&[b"pad", &xb, &yb], ctx.program_id);
            if account.key != &pda {
                panic!("wrong pad");
            }

            if account.data_is_empty() {
                let size = 10240;
                let reb = Rent::get()?.minimum_balance(size);
                let payer = ctx.accounts.owner.to_account_info();

                invoke_signed(
                    &system_instruction::create_account(
                        payer.key,
                        account.key,
                        reb,
                        size as u64,
                        ctx.program_id
                    ),
                    &[payer, account.clone()],
                    &[ &[b"pad", &xb, &yb, &[bump_seed]] ]
                )?;
            }

            add_cell_to_pad(account.data.borrow_mut(), ctx.accounts.beastie.cell_id, &pos);
        }
    }

    Ok(())
}

