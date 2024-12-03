use anchor_lang::prelude::*;
use beastie_common::Beastie;

use anchor_spl::{associated_token::AssociatedToken, token::{self, Mint, Token}};
use crate::{state::{beastie::{GridBeastie, Placement}, board::Board}, utils::*};

#[derive(Accounts)]
pub struct BillingContext<'info> {
    #[account(
        seeds = [b"grid.beastie", byte_ref!(asset_beastie.seed, 8)],
        bump,
        constraint = grid_beastie.placement_board.is_some()
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
        mut,
        seeds = [b"placement", asset_beastie.key().as_ref(), board.key().as_ref()],
        bump,
        constraint = placement.active.is_some()
    )]
    pub placement: Account<'info, Placement>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn bill_beastie<'info>(ctx: Context<'_, '_, '_, 'info, BillingContext<'info>>) -> Result<bool> {
    let height = Clock::get()?.slot;
    let mut p = ctx.accounts.placement.active.clone().unwrap();
    let diff = height - p.billed_height;
    if diff == 0 {
        return Ok(false);
    }

    let mut due = p.rate * diff;

    // first take from committed
    let take = if due > ctx.accounts.placement.committed { ctx.accounts.placement.committed } else { due };
    ctx.accounts.placement.committed -= take;
    due -= take;

    // then take from ATA
    let bal = ctx.accounts.beastie_ata.amount;
    let take = if due > bal { bal } else { due };
    if due > 0 {
        transfer_from_beastie_to_board(&ctx.accounts, take)?;
    }


    p.billed_height = height; // TODO: partial billing?
    ctx.accounts.placement.active = Some(p);

    Ok(true)
}

fn transfer_from_beastie_to_board(accounts: &BillingContext, amount: u64) -> Result<()> {
    let seeds = [BOARD_KEY, byte_ref!(accounts.board.seed, 8), &[accounts.board.bump]];
    let signer_seeds = [seeds.as_slice()];

    token::transfer(
        CpiContext::new_with_signer(
            accounts.token_program.to_account_info(),
            token::Transfer {
                from: accounts.beastie_ata.to_account_info(),
                to: accounts.board_ata.to_account_info(),
                authority: accounts.asset_beastie.to_account_info()
            },
            &signer_seeds
        ),
        amount
    )
}
