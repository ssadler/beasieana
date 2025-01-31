
pub mod common;

use common::*;
use anchor_lang::{prelude::*, solana_program::{sysvar::rent::Rent, sysvar::Sysvar}};
use anchor_spl::{associated_token::AssociatedToken, token::{self, Mint, Token}};
use beastie_common::{byte_ref, impl_deref, CELL_KEY};
use crate::state::beastie::*;


#[derive(Accounts)]
pub struct PlacementContext<'info> {
    pub c: PlacementCommon<'info>,
    #[account(
        seeds = [CELL_KEY, byte_ref!(c.beastie.cell_id, 4)],
        bump,
        mut
    )]
    pub cell: Account<'info, Cell>,
    #[account(
        associated_token::mint = c.token_mint,
        associated_token::authority = c.beastie,
        constraint = beastie_ata.key() != c.board_ata.key(),
        mut
    )]
    pub beastie_ata: Box<Account<'info, token::TokenAccount>>,
}

impl_deref!(['info], PlacementContext<'info>, PlacementCommon<'info>, c);

impl<'info> HasActiveBeastie for PlacementContext<'info> {
    fn get_cell(&self) -> &ActiveCell {
        &self.cell.as_active()
    }
    fn get_ata(&self) -> &token::TokenAccount {
        &self.beastie_ata
    }
}





#[derive(Accounts)]
pub struct InitPlacementContext<'info> {
    pub c: PlacementCommon<'info>,
    #[account(
        seeds = [CELL_KEY, byte_ref!(c.beastie.cell_id, 4)],
        bump
    )]
    pub cell: Account<'info, Cell>,
    #[account(
        init_if_needed,
        payer = c.payer,
        associated_token::mint = token_mint,
        associated_token::authority = c.beastie,
        constraint = beastie_ata.key() != c.board_ata.key()
    )]
    pub beastie_ata: Box<Account<'info, token::TokenAccount>>,
    pub token_mint: Account<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
impl_deref!(['info], InitPlacementContext<'info>, PlacementCommon<'info>, c);




