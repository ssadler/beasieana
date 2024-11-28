use std::str::FromStr;

use anchor_lang::prelude::*;

use crate::state::beastie::*;
use crate::state::global::*;




#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct InitBeastie<'info> {
    #[account(
        init,
        payer = payer,
        space = 4096,
        seeds = [b"grid.beastie", seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub grid_beastie: Account<'info, GridBeastie>,

    // This is required to authenticate that it's coming from the Beastie contract (it's a signer)
    #[account(
        seeds = [b"asset.beastie", seed.to_le_bytes().as_ref()],
        seeds::program = Pubkey::from_str("8Gg4bD4regjmpvz2thxNkyjvPiyxUKTcLuLZpFh4XJpU").unwrap(),
        bump
    )]
    pub asset_beastie: Signer<'info>,

    #[account(init_if_needed, payer = payer, space = 4096, seeds = [b"grid"], bump)]
    pub global: Account<'info, Global>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
