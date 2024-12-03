use std::{str::FromStr, u64};
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
use crate::{place_on_grid::place_beastie_on_grid, state::beastie::{ActivePlacement, GridBeastie, Placement}};
use crate::state::board::Board;
use crate::types::*;
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
}


pub fn place_beastie_on_board<'info>(ctx: Context<'_, '_, '_, 'info, PlacementContext<'info>>, pos: CellPos) -> Result<()> {
    // check min size, max size
    if pos.r < ctx.accounts.board.config.min_radius {
        panic!("min_radius");
    }
    if pos.r > ctx.accounts.board.config.max_radius {
        panic!("max_radius");
    }
    // check min value
    if ctx.accounts.beastie_ata.amount < ctx.accounts.board.config.add_cell_min_value {
        panic!("add_cell_min_value");
    }
    // Check placement is None
    if ctx.accounts.placement.active.is_some() {
        panic!("placement is active");
    }

    // Approve beastie for billing by board
    let approval = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Approve {
            to: ctx.accounts.beastie_ata.to_account_info(),
            delegate: ctx.accounts.board_ata.to_account_info(),
            authority: ctx.accounts.asset_beastie.to_account_info()
        }
    );
    token::approve(approval, u64::MAX)?;

    ctx.accounts.grid_beastie.placement_board = Some(ctx.accounts.board.key());
    ctx.accounts.placement.active = Some(ActivePlacement {
        billed_height: Clock::get()?.slot,
        rate: ctx.accounts.board.get_billing_rate(&pos),
        pos: pos.clone()
    });

    place_beastie_on_grid(
        ctx.program_id,
        ctx.remaining_accounts,
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.grid_beastie.cell_id,
        ctx.accounts.board.key(),
        pos
    )
}

