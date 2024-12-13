
use anchor_lang::prelude::*;

#[account]
pub struct Global {
    pub admin: Pubkey,
}


#[account]
pub struct GridTokenMeta {
    pub enabled: bool
}

