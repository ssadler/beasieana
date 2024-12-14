use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Token, TokenAccount, Mint}};
use beastie_common::*;
use crate::{remaining_accounts::CTX, state::{beastie::{Cell, HasActiveBeastie, Link}, board::Board}};



pub fn create_links<'c, 'info>(
    ctx: Context<'_, '_, 'c, 'info, LinksContext<'info>>,
    links: Vec<Link>
) -> Result<()> where 'c: 'info {
    let mut ctx = CTX::new(ctx);
    for link in links {
        create_link(&mut ctx, link)?;
    }
    Ok(())
}

const MAX_LINKS: usize = 10;

fn create_link<'c, 'info>(
    ctx: &mut CTX<'_, '_, 'c, 'info, LinksContext<'info>>,
    link: Link
) -> Result<()> where 'c: 'info {

    let config = &ctx.accounts.board.config;
    let cell = ctx.accounts.cell.as_active();
    let mut other_acct: Account<'info, Cell> = Account::try_from(
        ctx.rem.get_cell(link.cell_id, None).expect("error getting other cell in create_link")
    )?;
    let other = other_acct.as_active();

    assert!(cell.cell_id != link.cell_id, "is same cell wtf!!!");
    assert!(other.cell_id == link.cell_id, "cell id mismatch");
    assert!(cell.board == other.board, "board mismatch");
    let exists = cell.links.iter().position(|l| l.0.cell_id == link.cell_id).is_some();
    assert!(!exists, "already have link to same cell");
    assert!(cell.links.len() < MAX_LINKS, "too many links for cell");

    // check balance
    let bal = ctx.accounts.beastie_free_balance();
    assert!(
        bal > link.amount + config.add_cell_min_value,
        "low balance for link"
    );

    let distance = cell.pos.distance(&other.pos);
    let effectiveness = config.link_effectiveness(distance as u16);

    // at last, mutations!
    other_acct.as_active_mut().apply_link(&link, effectiveness);
    ctx.accounts.cell.links.push((link, effectiveness));

    Ok(())
}


pub fn remove_links<'c, 'info>(
    ctx: Context<'_, '_, 'c, 'info, LinksContext<'info>>,
    cell_ids: Vec<u32>
) -> Result<()> where 'c: 'info {
    let mut ctx = CTX::new(ctx);
    for id in cell_ids {
        remove_link(&mut ctx, id)?;
    }
    Ok(())
}

fn remove_link<'c, 'info>(
    ctx: &mut CTX<'_, '_, 'c, 'info, LinksContext<'info>>,
    cell_id: u32
) -> Result<()> where 'c: 'info {
    
    let mut other_acct: Account<'info, Cell> = Account::try_from(
        ctx.rem.get_cell(cell_id, None).expect("error getting other cell in create_link")
    )?;

    // To remove links we don't do so many checks
    let idx = ctx.accounts.cell.links.iter().position(|l| l.0.cell_id == cell_id)
        .expect("No such link");

    let (link, effectiveness) = ctx.accounts.cell.links.remove(idx);
    other_acct.as_active_mut().unapply_link(&link, effectiveness);

    Ok(())
}



#[derive(Accounts)]
pub struct LinksContext<'info> {
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
        seeds = [CELL_KEY, byte_ref!(cell.cell_id, 4)],
        bump,
        mut
    )]
    pub cell: Box<Account<'info, Cell>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = token_mint,
        associated_token::authority = beastie,
        constraint = beastie_ata.key() != board_ata.key()
    )]
    pub beastie_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        associated_token::mint = token_mint,
        associated_token::authority = board
    )]
    pub board_ata: Box<Account<'info, TokenAccount>>,
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


impl<'info> HasActiveBeastie for LinksContext<'info> {
    fn get_cell(&self) -> &crate::state::beastie::ActiveCell {
        self.cell.as_active()
    }
    fn get_ata(&self) -> &TokenAccount {
        &self.beastie_ata
    }
}
