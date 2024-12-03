use anchor_lang::prelude::*;

declare_id!("8Gg4bD4regjmpvz2thxNkyjvPiyxUKTcLuLZpFh4XJpU");

#[account]
pub struct Beastie {
    pub bump: u8,
    pub seed: u64,
    pub creation_slot: u64,
    pub owner: Pubkey
}


pub mod macros {
    macro_rules! byte_ref {
        ($val:expr, $size:expr) => {
            unsafe { &*(std::ptr::addr_of!($val) as *const [u8; $size]) }
        };
    }

    pub(crate) use byte_ref;    // <-- the trick
}

pub use macros::*;

use anchor_lang::prelude::Pubkey;


pub const BEASTIE_KEY: &[u8] = b"asset.beastie";
pub static BOARD_KEY: &[u8] = b"board";

pub static BEASTIE_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
  108,   5, 176, 35, 163, 168,  20, 119,
  211, 109,  62, 75,  73, 131, 136, 142,
  122,   8,  57, 49, 194,  77, 107,  29,
   82, 246,  77, 81, 218,  64, 238,   5
]);
