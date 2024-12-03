
use anchor_lang::prelude::*;

use crate::CellPos;

#[account]
pub struct GridBeastie {
    pub seed: u64,
    pub cell_id: u32,
    pub placement_board: Option<Pubkey>,
}


#[account]
pub struct Placement {
    pub committed: u64,
    pub active: Option<ActivePlacement>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ActivePlacement {
    pub pos: CellPos,
    pub billed_height: u64,
    pub rate: u64,
}

