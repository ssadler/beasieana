
use anchor_lang::prelude::*;

declare_id!("8Gg4bD4regjmpvz2thxNkyjvPiyxUKTcLuLZpFh4XJpU");

#[account]
pub struct Beastie {
    pub cell_id: u32,
    pub creation_slot: u64,
    pub owner: Pubkey
}

pub mod macros {
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
    macro_rules! impl_deref {
        ( [$($impl_generics:tt)*], $type:ty, $target:ty, $field:tt) => {
            $crate::impl_deref_const!([$($impl_generics)*], $type, $target, $field);

            impl<$($impl_generics)*> std::ops::DerefMut for $type {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.$field
                }
            }
        };
    }

    #[macro_export]
    macro_rules! impl_deref_const {
        ( [$($impl_generics:tt)*], $type:ty, $target:ty, $field:tt) => {
            impl<$($impl_generics)*> std::ops::Deref for $type {
                type Target = $target;
                fn deref(&self) -> &Self::Target {
                    &self.$field
                }
            }
        }
    }

    pub use byte_ref;
    pub use leak;
    pub use impl_deref;
    pub use impl_deref_const;
}

pub use macros::*;


use anchor_lang::prelude::Pubkey;


pub static BEASTIE_KEY:          &[u8] = b"beastie";
pub static CELL_KEY:             &[u8] = b"cell";
pub static BOARD_KEY:            &[u8] = b"board";
pub static PAD_KEY:              &[u8] = b"pad";


pub static BEASTIE_PROGRAM_ID: Pubkey = pubkey!("8Gg4bD4regjmpvz2thxNkyjvPiyxUKTcLuLZpFh4XJpU");
pub static GRID_PROGRAM_ID: Pubkey = pubkey!("EExeRoQMrfcJP28XQVjcE6khh3U8GC2RVZs28RNut5Br");


