use anchor_lang::prelude::*;
use beastie_common::{BEASTIE_KEY, CELL_KEY, BEASTIE_PROGRAM_ID};

use crate::state::beastie::*;




#[derive(Accounts)]
#[instruction(cell_id: u32)]
pub struct InitBeastie<'info> {
    #[account(
        init,
        payer = payer,
        space = 10240,
        seeds = [CELL_KEY, cell_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub placement: Account<'info, Cell>,

    // This is required to authenticate that it's coming from the Beastie contract (it's a signer)
    #[account(
        seeds = [BEASTIE_KEY, cell_id.to_le_bytes().as_ref()],
        seeds::program = BEASTIE_PROGRAM_ID,
        bump
    )]
    pub beastie: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
