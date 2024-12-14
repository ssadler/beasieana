use anchor_lang::prelude::*;

declare_id!("8Gg4bD4regjmpvz2thxNkyjvPiyxUKTcLuLZpFh4XJpU");

#[account]
pub struct Beastie {
    pub bump: u8,
    pub cell_id: u32,
    pub creation_slot: u64,
    pub owner: Pubkey
}

#[macro_export]
macro_rules! byte_ref {
    ($val:expr, $size:expr) => {
        unsafe { &*(std::ptr::addr_of!($val) as *const [u8; $size]) }
    };
}

#[macro_export]
macro_rules! leak {
    ($val:expr) => {
        Box::leak(Box::new($val))
    };
}

#[macro_export]
macro_rules! u8_to_vec_box {
    ( $( $arr:expr ),* ) => {
        vec![
            $( Box::from($arr.clone()) ),*
        ]
    };
}


use anchor_lang::prelude::Pubkey;


pub static BEASTIE_KEY:          &[u8] = b"beastie";
pub static CELL_KEY:             &[u8] = b"cell";
pub static BOARD_KEY:            &[u8] = b"board";
pub static PAD_KEY:              &[u8] = b"pad";


pub static BEASTIE_PROGRAM_ID: Pubkey = pubkey!("8Gg4bD4regjmpvz2thxNkyjvPiyxUKTcLuLZpFh4XJpU");
pub static GRID_PROGRAM_ID: Pubkey = pubkey!("EExeRoQMrfcJP28XQVjcE6khh3U8GC2RVZs28RNut5Br");


