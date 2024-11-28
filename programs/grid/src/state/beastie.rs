
use anchor_lang::prelude::*;

#[account]
pub struct GridBeastie {
    pub seed: u64,
    pub cell_id: u32
}
