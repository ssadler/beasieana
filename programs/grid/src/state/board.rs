use anchor_lang::prelude::*;

use crate::CellPos;


#[account]
pub struct Board {
    pub bump: u8,
    pub seed: u64,
    pub token: Pubkey,
    pub owner: Pubkey,
    pub config: BoardConfig,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct BoardConfig {
    pub rate: u64,
    pub add_cell_min_value: u64,
    pub min_radius: u16,
    pub max_radius: u16,
    pub width: u16,
    pub height: u16
}

impl Board {
    pub fn create(&mut self, config: BoardConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }

    pub fn get_billing_rate(&self, pos: &CellPos) -> u64 {
        return self.config.rate * pos.area()
    }
}

impl BoardConfig {
    pub fn contains_circle(&self, pos: &CellPos) -> bool {
        // Check lower bounds
        if pos.x < pos.r || pos.y < pos.r {
            false
        } else if self.width - pos.r < pos.x || self.height - pos.r < pos.y {
            false
        } else {
            true
        }
    }
}

