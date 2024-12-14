
use std::ops::Deref;

use anchor_lang::prelude::*;
use anchor_spl::token;

use crate::{remaining_accounts::CTX, state::beastie::{ActiveCell, HasActiveBeastie}, CellPos, CellPositionedId};

use super::PlacementContext;




pub fn interact<'info>(
    ctx: &mut CTX<'_, '_, '_, 'info, PlacementContext<'info>>,
    pos: &CellPos,
    other: &CellPositionedId
) -> Result<CellPos> 
{
    let config = &ctx.accounts.board.config;

    let new_pos = shift_shrink(pos, &other.pos).expect("OCCUPIED");
    if new_pos.r < config.min_radius {
        panic!("OCCUPIED");
    }

    let other_beastie: Account<'info, ActiveCell> = Account::try_from(
        ctx.rem.get_placement(other.cell_id, None).expect("error getting placement PDA in interact")
    )?;
    let other_ata: Account<'info, token::TokenAccount> = Account::try_from(
        ctx.rem.get_ata(&other_beastie.asset_address(), &ctx.accounts.board.token).expect("Error getting ATA in interact")
    )?;

    let sec = ctx.accounts.beastie_security_balance()?;
    let other_sec = (&*other_beastie, &*other_ata).beastie_security_balance()?;

    if sec < other_sec {
        panic!("low balance for shrink");
    }

    // commit displacement
    let area_diff = other.pos.area() - new_pos.area();
    panic!("displacement");
    //let displacement = other_sec * area_diff / other.pos.area();
    //ctx.commit_balance(displacement)?;

    Ok(new_pos)
}


pub fn shift_shrink(pos: &CellPos, other: &CellPos) -> Option<CellPos> {

    if pos.contains(other) {
        return None;
    }

    let vx = other.x as i32;
    let vy = other.y as i32;
    let ax = pos.x as i32;
    let mut ay = pos.y as i32;
    // possibly shift off center
    if vx == ax && vy == ay { ay += 1 };

    // distance between cells
    let dx = (vx as f32) - (ax as f32);
    let dy = (vy as f32) - (ay as f32);
    let distance = (dx*dx+dy*dy).sqrt();

    let rad_sum = (pos.r + other.r) as f32;

    let mut shift = 0.0;
    if distance < rad_sum {
        shift = (rad_sum - distance) / 2.0;
    }

    Some(
        CellPos {
            x: (vx + ((dx * shift) / distance) as i32) as u16,
            y: (vy + ((dy * shift) / distance) as i32) as u16,
            r: other.r - (shift as u16)
        }
    )
}






#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift_shrink() {
        let old = &CellPos { x: 100, y: 100, r: 100 };
        let new = &CellPos { x: 200, y: 100, r: 100 };

        let r = shift_shrink(new, old).unwrap();

        msg!("r: {:?}", r);
        assert!(r == CellPos { x: 50, y: 100, r: 50 });
        assert!(r.overlaps(new) == false);


    }
}













//pub fn simple_shrink(pos: &CellPos, other: &CellPos) -> Option<CellPos> {
//    let vx = other.x;
//    let vy = other.y;
//    let ax = pos.x;
//    let ay = pos.y;
//
//    // distance between cells
//    let dx = (vx as f32) - (ax as f32);
//    let dy = (vy as f32) - (ay as f32);
//    let distance = (dx*dx+dy*dy).sqrt() as u16;
//
//    if distance >= other.r {
//        None
//    } else {
//        Some(CellPos { x: other.x, y: other.y, r: other.r - distance })
//    }
//
//}


//function shiftShrink(Cell memory aggCell, Cell memory vicCell)
//  pure returns (CellCoord memory coord)
//{
//  /*
//   * This one is harder, shrink and move the cell in order to make
//   * better use of space
//   */
//
//  uint scale = 1e18;
//
//  if (Geometry.cellContains(aggCell, vicCell)) {
//    return coord; // return 0,0,0
//  }
//
//  unchecked {
//    // Resolution I guess?
//    // v = victim, a = aggressor
//    uint vx = vicCell.x * scale;
//    uint vy = vicCell.y * scale;
//    uint ax = aggCell.x * scale;
//    uint ay = aggCell.y * scale;
//
//    if (vx == ax && vy == ay) {
//      vy += 1; // Shift it off center, otherwise which way does it go?
//    }
//
//    // d=distance between cells
//    int dx = int(uint(vx)) - int(uint(ax));
//    int dy = int(uint(vy)) - int(uint(ay));
//    uint distance = Math.sqrt(uint(dx*dx+dy*dy));
//
//    // sum of the radii
//    uint totRad = (uint(aggCell.r) + uint(vicCell.r)) * scale;
//
//    // the shift distance is half the overlap, which fits when r is reduced by the distance
//    uint shift = 0;
//    if (distance <= totRad) {
//      shift = (totRad - distance) / 2;
//    }
//
//    coord.x = uint24(uint(int(vx) + (dx * int(shift)) / int(distance)) / scale);
//    coord.y = uint24(uint(int(vy) + (dy * int(shift)) / int(distance)) / scale);
//    coord.r = uint16((uint(vicCell.r) * scale - shift) / scale);
//  }
//}
