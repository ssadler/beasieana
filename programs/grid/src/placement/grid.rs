
use anchor_lang::prelude::*;
use crate::remaining_accounts::{InitPDA, RemainingAccounts};
use crate::state::pad;
use crate::{types::*, BillingContext};
use crate::placement::context::*;

use super::interaction::interact;



const RAD_MAX: u16 = 500;

pub fn place_beastie_on_grid<'info>(
    ctx: &mut Context<'_, '_, '_, 'info, PlacementContext<'info>>,
    vra: &mut RemainingAccounts<'info>,
    cell: CellPositionedId
) -> Result<()> {


    if cell.pos.r > RAD_MAX {
        panic!("RAD_MAX is 500");
    }
    if !ctx.accounts.board.config.contains_circle(&cell.pos) {
        panic!("OOB");
    }

    let mut interacted = false;

    for pad_id in cell.pos.pads(9) {
        let init = Some((&ctx.accounts.payer, 10240));
        let mut pad = load_pad_storage(vra, pad_id, init);
        let mut idx = 0;
        while idx < pad.len() {
            let other = pad.get(idx);
            if cell.pos.overlaps(&other.pos) {
                if interacted {
                    msg!("already interacted: {:?}", other.pos);
                    panic!("already interacted");
                }
                interacted = true;
                replace_interact(ctx, vra, &cell, other.clone())?;
            } else {
                idx += 1;
            }
        }
        pad.append(ctx.accounts.beastie.cell_id, &cell.pos)
    }

    Ok(())
}


fn load_pad_storage<'info>(
    vra: &RemainingAccounts<'info>,
    (xx, yy): (u16, u16),
    init: InitPDA<'_, 'info>,
) -> pad::PadStorage<'info> {
    let pad_pda = vra
        .get_pad(b"", xx, yy, init)
        .expect("error getting pad in place");
    pad::PadStorage::new(pad_pda.data.clone())
}


fn replace_interact<'c, 'info>(
    ctx: &mut Context<'_, '_, 'c, 'info, PlacementContext<'info>>,
    vra: &RemainingAccounts<'info>,
    cell: &CellPositionedId,
    mut other: CellPositionedId
) -> Result<()> {

    if cell.cell_id != ctx.get_placement().cell_id {
        panic!("unexpected interact");
    }

    let old_pos = other.pos.clone();
    other.pos = interact(ctx, vra, &cell.pos, &other)?;

    for pad in old_pos.pads(9) {
        let mut storage = load_pad_storage(vra, pad, None);
        if other.pos.overlaps_pad(pad, 9) {
            storage.update_cell(&other);
        } else {
            storage.remove_cell(other.cell_id);
        }
    };

    Ok(())
}






pub fn remove_beastie_from_grid<'info>(
    vra: &mut RemainingAccounts<'info>,
    cell: CellPositionedId,
) -> Result<()> {

    for (xx, yy) in cell.pos.pads(9) {
        let pad = vra.get_pad(b"", xx, yy, None).expect("Error getting Pad in remove");
        if !pad.data_is_empty() {
            pad::PadStorage::new(pad.data.clone()).remove_cell(cell.cell_id);
        }
    }

    Ok(())
}

