use anchor_lang::prelude::*;

#[account]
pub struct Board {
    pub config: BoardConfig,
    pub prev_cell_id: u32
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct BoardConfig {
    pub rental_price: u32,
    pub width: u32,
    pub height: u32
}

impl Board {
    pub fn create(&mut self, config: BoardConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
}
