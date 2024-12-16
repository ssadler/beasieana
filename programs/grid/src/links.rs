use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Token, TokenAccount, Mint}};
use beastie_common::*;
use crate::{remaining_accounts::CTX, state::beastie::{Cell, EffectiveLink, HasActiveBeastie, Link}, PlacementContext};



const MAX_LINKS: usize = 10;


pub fn create_link<'info>(
    ctx: &mut CTX<'_, '_, '_, 'info, PlacementContext<'info>>,
    link: Link
) -> Result<()> {

    let config = &ctx.accounts.board.config;
    let cell = &ctx.accounts.cell.as_active();
    let mut other_acct: Account<'info, Cell> = Account::try_from(
        ctx.rem.get_cell(link.cell_id, None).expect("error getting other cell in create_link")
    )?;
    let other = other_acct.as_active();

    assert!(cell.cell_id != link.cell_id, "is same cell wtf!!!");
    assert!(other.cell_id == link.cell_id, "cell id mismatch");
    assert!(cell.board == other.board, "board mismatch");
    let exists = cell.links.iter().position(|l| l.link.cell_id == link.cell_id).is_some();
    assert!(!exists, "already have link to same cell");
    assert!(cell.links.len() < MAX_LINKS, "too many links for cell");

    // check balance
    assert!(
        ctx.accounts.beastie_free_balance() > link.amount + config.add_cell_min_value,
        "low balance for link"
    );

    let distance = cell.pos.distance(&other.pos);
    let effectiveness = config.link_effectiveness(distance as u16);
    let elink = EffectiveLink { link, effectiveness };

    // at last, mutations!
    other_acct.as_active_mut().apply_link(&elink);
    ctx.accounts.cell.links.push(elink);

    Ok(())
}



pub fn remove_link<'info>(
    ctx: &mut CTX<'_, '_, '_, 'info, PlacementContext<'info>>,
    cell_id: u32
) -> Result<()> {
    
    let mut other_acct: Account<'info, Cell> = Account::try_from(
        ctx.rem.get_cell(cell_id, None).expect("error getting other cell in create_link")
    )?;

    // To remove links we don't do so many checks
    let idx = ctx.accounts.cell.links.iter()
        .position(|l| l.link.cell_id == cell_id)
        .expect("No such link");

    let elink = ctx.accounts.cell.links.remove(idx);
    other_acct.as_active_mut().unapply_link(&elink);

    Ok(())
}

