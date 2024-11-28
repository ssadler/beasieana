use anchor_lang::prelude::*;

use crate::state::board::*;
use crate::state::global::Global;


#[derive(Accounts)]
#[instruction(seed: u64, owner: Pubkey, config: BoardConfig)]
pub struct CreateBoard<'info> {
    #[account(
        init,
        payer = payer,
        space = 4096,
        seeds = [b"board", seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub board: Account<'info, Board>,

    #[account(mut, seeds = [b"grid"], bump)]
    pub global: Account<'info, Global>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
