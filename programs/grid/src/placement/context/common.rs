
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token};
use beastie_common::{byte_ref, Beastie, BEASTIE_KEY, BEASTIE_PROGRAM_ID};
use crate::state::board::Board;



#[derive(Accounts)]
pub struct PlacementCommon<'info> {

    // This is required to authenticate that it's coming from the Beastie contract (it's a signer)
    #[account(
        signer,
        seeds = [BEASTIE_KEY, byte_ref!(beastie.cell_id, 4)],
        seeds::program = BEASTIE_PROGRAM_ID,
        bump
    )]
    pub beastie: Box<Account<'info, Beastie>>,

    #[account(
        seeds = [b"board", board.seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub board: Box<Account<'info, Board>>,

    pub token_mint: Account<'info, Mint>,

    #[account(mut,
        associated_token::mint = token_mint,
        associated_token::authority = board
    )]
    pub board_ata: Box<Account<'info, token::TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub payer: Signer<'info>,
}

