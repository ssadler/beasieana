
mod context;
mod grid;
mod interaction;

pub use context::*;
pub use grid::*;
use crate::billing;

use anchor_lang::prelude::*;

use crate::{remaining_accounts::RemainingAccounts, start_billing, BillingContext, CellPos};


pub fn place<'c, 'info>(mut ctx: Context<'_, '_, 'c, 'info, PlacementContext<'info>>, pos: CellPos) -> Result<()> where 'c: 'info {

    let mut vra = RemainingAccounts::new(ctx.remaining_accounts);
    start_billing(&mut ctx, pos)?;
    let cell = crate::CellPositionedId { cell_id: ctx.get_placement().cell_id, pos };
    place_beastie_on_grid(&mut ctx, &mut vra, cell)
}


pub fn remove<'c, 'info>(mut ctx: Context<'_, '_, 'c, 'info, PlacementContext<'info>>) -> Result<()> where 'c: 'info {
    // TODO: Bill
    let mut vra = RemainingAccounts::new(ctx.remaining_accounts);
    let cell = ctx.get_placement().get_active().get_cell();
    remove_beastie_from_grid(&mut vra, cell)
}


pub fn bill<'c, 'info>(&mut ctx: Context<'_, '_, 'c, 'info, PlacementContext<'info>>) -> Result<()> where 'c: 'info {
    let r = billing::bill_beastie(ctx);
    if r == billing::BillingResult::Broke {
        let cell = ctx.get_placement().get_active().get_cell();
        remove_beastie_from_grid(&mut vra, cell)


    }
}
