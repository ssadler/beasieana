use std::ops::Deref;

use anchor_lang::prelude::*;
use beastie_common::{byte_ref, BEASTIE_KEY, BEASTIE_PROGRAM_ID};

use crate::CellPos;
use crate::batteries::*;
use crate::CellPositionedId;


#[account]
pub struct GridBeastie {
    pub cell_id: u32,
    pub active: Option<Placement>,
    pub commitments: Vec<Commitment>,
}

impl GridBeastie {
    pub fn asset_address(&self) -> Pubkey {
        let seeds = [BEASTIE_KEY, byte_ref!(self.cell_id, 4)];
        let (addr, _) = Pubkey::find_program_address(&seeds, &BEASTIE_PROGRAM_ID);
        addr
    }
    pub fn get_active<'b>(&'b self) -> ActiveCell<'b> {
        ActiveCell {
            cell_id: self.cell_id,
            commitments: &self.commitments,
            placement: self.active.as_ref().expect("get_active: placement not active")
        }
    }
}

pub struct ActiveCell<'b> {
    pub cell_id: u32,
    pub commitments: &'b Vec<Commitment>,
    placement: &'b Placement
}

impl<'b> Deref for ActiveCell<'b> {
    type Target = Placement;
    fn deref(&self) -> &Self::Target {
        &self.placement
    }
}

impl<'b> ActiveCell<'b> {
    pub fn get_cell(&self) -> CellPositionedId {
        CellPositionedId { cell_id: self.cell_id, pos: self.pos }
    }
}





#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct Placement {
    pub board: Pubkey,
    pub pos: CellPos,
    pub billed_height: u64,
    pub rate: u64,
}

impl Placement {
    pub fn get_due(&self) -> Result<u64> {
        let height = Clock::get()?.slot;
        Ok((height - self.billed_height) * self.rate)
    }
}
