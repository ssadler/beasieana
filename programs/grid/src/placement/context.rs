use anchor_lang::{
    prelude::*,
    solana_program::{
        pubkey::Pubkey,
        sysvar::rent::Rent,
        sysvar::Sysvar,
    }
};
use anchor_spl::{associated_token::AssociatedToken, token::{self, Mint, Token}};
use beastie_common::Beastie;
use signertest::program::Signertest;
use signertest2::program::Signertest2;
use crate::state::beastie::{GridBeastie, Placement};
use crate::state::board::Board;
use crate::utils::*;


#[derive(Accounts)]
pub struct PlacementContext<'info> {
    #[account(
        seeds = [b"grid.beastie", byte_ref!(asset_beastie.seed, 8)],
        bump,
        constraint = grid_beastie.placement_board.is_none()
    )]
    pub grid_beastie: Account<'info, GridBeastie>,

    // This is required to authenticate that it's coming from the Beastie contract (it's a signer)
    #[account(
        signer,
        seeds = [b"asset.beastie", byte_ref!(asset_beastie.seed, 8)],
        seeds::program = BEASTIE_PROGRAM_ID,
        bump
    )]
    pub asset_beastie: Account<'info, Beastie>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_mint,
        associated_token::authority = asset_beastie
    )]
    pub beastie_ata: Account<'info, token::TokenAccount>,
    #[account(mut,
        associated_token::mint = token_mint,
        associated_token::authority = board
    )]
    pub board_ata: Account<'info, token::TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(constraint = token_mint.key() == board.token)]
    pub token_mint: Account<'info, Mint>,

    #[account(
        seeds = [b"board", board.seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub board: Account<'info, Board>,

    #[account(
        init_if_needed,
        space = 1024,
        payer = payer,
        seeds = [b"placement", asset_beastie.key().as_ref(), board.key().as_ref()],
        bump,
    )]
    pub placement: Account<'info, Placement>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    //pub signertest_program: Program<'info, Signertest>,
    //pub signertest2_program: Program<'info, Signertest2>,
}


