
use anchor_lang::prelude::*;

#[account]
pub struct Global {
    pub admin: Pubkey,
    prev_cell_id: u32,
}

impl Global {
    pub fn next_cell_id(&mut self) -> u32 {
        self.prev_cell_id += 1;
        self.prev_cell_id
    }
}


#[account]
pub struct GridTokenMeta {
    pub enabled: bool
}

