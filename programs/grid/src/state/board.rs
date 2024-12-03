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
    pub width: u32,
    pub height: u32
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
