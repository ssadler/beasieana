
mod context;
mod grid;
mod interaction;

use std::ops::DerefMut;

pub use context::*;
pub use grid::*;
use crate::{billing, remaining_accounts::CTX};

use anchor_lang::prelude::*;

use crate::{start_billing, BillingContext, CellPos};


pub fn place<'c, 'info>(ctx: Context<'_, '_, 'c, 'info, PlacementContext<'info>>, pos: CellPos) -> Result<()> where 'c: 'info {
    let mut ctx = CTX::new(ctx);
    start_billing(ctx.deref_mut(), pos)?;
    let cell = crate::CellPositionedId { cell_id: ctx.get_cell().cell_id, pos };
    place_beastie_on_grid(&mut ctx, cell)
}


pub fn remove<'c, 'info>(ctx: Context<'_, '_, 'c, 'info, PlacementContext<'info>>) -> Result<()> where 'c: 'info {
    let mut ctx = CTX::new(ctx);
    _remove(&mut ctx)?;
    billing::stop_billing(ctx.deref_mut());
    Ok(())
}


pub fn bill_or_remove<'c, 'info>(ctx: &mut CTX<'_, '_, 'c, 'info, PlacementContext<'info>>) -> Result<billing::BillingResult> where 'c: 'info {
    let r = billing::bill_beastie(ctx.deref_mut())?;
    if r == billing::BillingResult::Broke {
        _remove(ctx);
    }
    Ok(r)
}


fn _remove<'c, 'info>(ctx: &mut CTX<'_, '_, 'c, 'info, PlacementContext<'info>>) -> Result<()> where 'c: 'info {
    let cell = ctx.get_cell().as_active().get_cell();
    remove_beastie_from_grid(&mut ctx.rem, cell)
}

