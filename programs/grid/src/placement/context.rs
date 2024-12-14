
use anchor_lang::{
    prelude::*,
    solana_program::{
        pubkey::Pubkey,
        sysvar::rent::Rent,
        sysvar::Sysvar,
    }
};
use anchor_spl::{associated_token::AssociatedToken, token::{self, Mint, Token}};
use beastie_common::{byte_ref, Beastie, BEASTIE_KEY, BEASTIE_PLACEMENT, BEASTIE_PROGRAM_ID};
use crate::state::beastie::*;
use crate::state::board::Board;
use crate::billing::BillingContext;



#[derive(Accounts)]
pub struct PlacementContext<'info> {
    // This is required to authenticate that it's coming from the Beastie contract (it's a signer)
    #[account(
        signer,
        seeds = [BEASTIE_KEY, byte_ref!(beastie.cell_id, 4)],
        seeds::program = BEASTIE_PROGRAM_ID,
        bump,
        mut
    )]
    pub beastie: Box<Account<'info, Beastie>>,

    #[account(
        seeds = [BEASTIE_PLACEMENT, byte_ref!(placement.cell_id, 4)],
        bump,
        mut
    )]
    pub placement: Box<Account<'info, Cell>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_mint,
        associated_token::authority = beastie,
        constraint = beastie_ata.key() != board_ata.key()
    )]
    pub beastie_ata: Box<Account<'info, token::TokenAccount>>,
    #[account(mut,
        associated_token::mint = token_mint,
        associated_token::authority = board
    )]
    pub board_ata: Box<Account<'info, token::TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(constraint = token_mint.key() == board.token)]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        seeds = [b"board", board.seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub board: Box<Account<'info, Board>>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'a, 'b, 'c, 'info> BillingContext<'info> for Context<'a, 'b, 'c, 'info, PlacementContext<'info>> {
    fn get_placement(&mut self) -> &mut Account<'info, Cell> {
        &mut self.accounts.placement
    }
    fn beastie_ata(&self) -> &Account<'info, token::TokenAccount> {
        &self.accounts.beastie_ata
    }
    fn get_beastie(&self) -> &Account<'info, Beastie> {
        &self.accounts.beastie
    }
    fn billing_board(&self) -> &Account<'info, Board> {
        &self.accounts.board
    }
    fn board_ata(&self) -> &Account<'info, token::TokenAccount> {
        &self.accounts.board_ata
    }
    fn billing_token_program(&self) -> AccountInfo<'info> {
        self.accounts.token_program.to_account_info()
    }
}

impl<'info> HasActiveBeastie for PlacementContext<'info> {
    fn get_cell(&self) -> &ActiveCell {
        self.placement.as_active()
    }
    fn get_ata(&self) -> &token::TokenAccount {
        &self.beastie_ata
    }
}
