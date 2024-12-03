use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint};
use anchor_spl::associated_token::AssociatedToken;

use crate::state::board::*;
use crate::state::global::*;


#[derive(Accounts)]
#[instruction(seed: u64, owner: Pubkey, token: Pubkey, config: BoardConfig)]
pub struct CreateBoard<'info> {
    #[account(
        init,
        payer = payer,
        space = 4096,
        seeds = [b"board", seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub board: Account<'info, Board>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = token_mint,
        associated_token::authority = board
    )]
    pub board_ata: Account<'info, token::TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(constraint = token_mint.key() == token)]
    pub token_mint: Account<'info, Mint>,

    #[account(
        seeds = [b"token_meta", token.as_ref()], bump,
        constraint = grid_token_meta.enabled
    )]
    pub grid_token_meta: Account<'info, GridTokenMeta>,

    #[account(mut, seeds = [b"grid"], bump)]
    pub global: Account<'info, Global>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
