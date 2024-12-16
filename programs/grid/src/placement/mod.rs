
pub mod context;
mod grid;
mod interaction;

pub use context::*;
pub use grid::*;
use crate::{billing, remaining_accounts::CTX};

use anchor_lang::prelude::*;

use crate::{CellPos};


pub fn place<'c, 'info>(
    ctx: Context<'_, '_, 'c, 'info, PlacementContext<'info>>,
    pos: CellPos
) -> Result<()> where 'c: 'info {
    let mut ctx = CTX::new(ctx);
    ctx.accounts.start_billing(pos)?;
    let cell = crate::CellPositionedId { cell_id: ctx.accounts.cell.cell_id, pos };
    place_beastie_on_grid(&mut ctx, cell)
}


pub fn remove<'c, 'info>(ctx: Context<'_, '_, 'c, 'info, PlacementContext<'info>>) -> Result<()> where 'c: 'info {
    let mut ctx = CTX::new(ctx);
    _remove(&mut ctx)?;
    billing::stop_billing(ctx.accounts)?;
    Ok(())
}


pub fn bill_or_remove<'c, 'info>(ctx: &mut CTX<'_, '_, 'c, 'info, PlacementContext<'info>>) -> Result<billing::BillingResult> where 'c: 'info {
    let r = ctx.accounts.bill_beastie()?;
    if r == billing::BillingResult::Broke {
        _remove(ctx)?;
    }
    Ok(r)
}


fn _remove<'c, 'info>(ctx: &mut CTX<'_, '_, 'c, 'info, PlacementContext<'info>>) -> Result<()> where 'c: 'info {
    let cell = ctx.accounts.cell.as_active().get_cell();
    remove_beastie_from_grid(&mut ctx.rem, cell)
}

