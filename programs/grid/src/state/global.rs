
use anchor_lang::prelude::*;

#[account]
pub struct Global {
    prev_cell_id: u32,
}

impl Global {
    pub fn next_cell_id(&mut self) -> u32 {
        self.prev_cell_id += 1;
        self.prev_cell_id
    }
}
