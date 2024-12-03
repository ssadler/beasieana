
pub mod macros {
    macro_rules! byte_ref {
        ($val:expr, $size:expr) => {
            unsafe { &*(std::ptr::addr_of!($val) as *const [u8; $size]) }
        };
    }

    pub(crate) use byte_ref;    // <-- the trick
}

pub use macros::*;



pub static BEASTIE_KEY: &[u8] = b"asset.beastie";
