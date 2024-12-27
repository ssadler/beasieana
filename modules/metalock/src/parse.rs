
use std::collections::{BTreeMap};

use anchor_lang::prelude::*;

use crate::types::*;



pub struct Parser<'a, 'b>(pub(crate) &'a mut &'b [u8]);
pub type R<T> = std::result::Result<T, String>;




#[inline]
fn many<I: IntoIterator, T, F: FnMut(&I::Item) -> R<T>>(n: I, mut f: F) -> R<Vec<T>> {
    n.into_iter().map(|a| f(&a)).collect::<R<Vec<_>>>()
}

pub fn take<const T: usize>(buf: Buf) -> R<[u8; T]> {
    let o = TryInto::<[u8; T]>::try_into(&buf[..T]).map_err(|s| s.to_string())?;
    *buf = &buf[T..];
    Ok(o)
}

#[inline]
fn rdd<T: ResourceDataDeserialize>(buf: Buf) -> R<T> {
    ResourceDataDeserialize::rd_deserialize(buf)
}


mod macros {

    #[macro_export]
    macro_rules! impl_deserialize_any_generic {
        (|$t:tt| $type:ty, |$buf:ident| $process:expr) => {
            impl<$t: ResourceDataDeserialize> ResourceDataDeserialize for $type {
                fn rd_deserialize($buf: Buf) -> R<$type> {
                    $process
                }
            }
        }
    }
    pub use impl_deserialize_any_generic;

    #[macro_export]
    macro_rules! impl_deserialize_any {
        ($type:ty, |$buf:ident| $process:expr) => {
            impl ResourceDataDeserialize for $type {
                fn rd_deserialize($buf: Buf) -> R<$type> {
                    $process
                }
            }
        };
    }
    pub use impl_deserialize_any;
    #[macro_export]
    macro_rules! impl_deserialize_int {
        ($type:tt) => {
            crate::impl_deserialize_any!($type, |buf| Ok($type::from_le_bytes(take(buf)?)));
        };
    }
    pub use impl_deserialize_int;
}

macros::impl_deserialize_int!(u8);
macros::impl_deserialize_int!(u16);
macros::impl_deserialize_int!(u32);
macros::impl_deserialize_int!(u64);
macros::impl_deserialize_int!(u128);
macros::impl_deserialize_any!(bool, |buf| Ok(u8::rd_deserialize(buf)? > 0));
macros::impl_deserialize_any!(Pubkey, |buf| Ok(Pubkey::from(take(buf)?)));
macros::impl_deserialize_any_generic!(|T| Option<T>, |buf| Option::rd_many(buf, rdd));
macros::impl_deserialize_any_generic!(|T| (String, T), |buf| Ok((rdd(buf)?, rdd(buf)?)));
macros::impl_deserialize_any_generic!(|T| Vec<T>, |buf| Vec::rd_many(buf, rdd));
macros::impl_deserialize_any!(String, |buf| {
    let Buffer(v) = Buffer::rd_deserialize(buf)?;
    Ok(String::from_utf8(v).map_err(|s| s.to_string())?)
});
macros::impl_deserialize_any!(Buffer, |buf| {
    let len: u16 = rdd(buf)?;
    let (a, rest) = buf.split_at(len as usize);
    *buf = rest;
    Ok(Buffer(a.to_vec()))
});





/*
 * Is this trait worth it?
 */
pub trait ResourceDataContainer<T>: Sized {
    fn rd_many<F: FnMut(Buf) -> R<T>>(buf: Buf, f: F) -> R<Self>;
}

impl<T> ResourceDataContainer<T> for Vec<T> {
    fn rd_many<F: FnMut(Buf) -> R<T>>(buf: Buf, mut f: F) -> R<Self> {
        many(0..u16::rd_deserialize(buf)?, |_| f(buf))
    }
}
impl<T> ResourceDataContainer<T> for BTreeMap<String, T> {
    fn rd_many<F: FnMut(Buf) -> R<T>>(buf: Buf, mut f: F) -> R<Self> {
        let iter = 0..u16::rd_deserialize(buf)?;
        let ff = |_| Ok((rdd::<String>(buf)?, f(buf)?));
        iter.into_iter().map(ff).collect()
    }
}
impl<T> ResourceDataContainer<T> for Option<T> {
    fn rd_many<F: FnMut(Buf) -> R<T>>(buf: Buf, mut f: F) -> R<Self> {
        if u8::rd_deserialize(buf)? > 0 {
            Ok(Some(f(buf)?))
        } else {
            Ok(None)
        }
    }
}



pub fn rdd_deserialize(rs: &ResourceSchema, buf: Buf) -> R<ResourceData> {
    Ok(match &rs.0 {
        RS::U8 => RD::U8(rdd(buf)?),
        RS::U16 => RD::U16(rdd(buf)?),
        RS::U32 => RD::U32(rdd(buf)?),
        RS::U64 => RD::U64(rdd(buf)?),
        RS::U128 => RD::U128(rdd(buf)?),
        RS::Bool => RD::Bool(rdd(buf)?),
        RS::Buffer => RD::Buffer(rdd(buf)?),
        RS::String => RD::String(rdd(buf)?),
        RS::Pubkey => RD::Pubkey(rdd(buf)?),
        RS::Option(rs) => RD::Option(Option::rd_many(buf, |buf| rdd_deserialize(rs, buf))?.into()),
        RS::Map(rs) => RD::Map(BTreeMap::rd_many(buf, |buf| rdd_deserialize(rs, buf))?),
        RS::List(rs) => RD::List(Vec::rd_many(buf, |buf| rdd_deserialize(rs, buf))?),
        RS::Struct(defs) => RD::Struct(many(defs, |(k, rs)| Ok((k.clone(), rdd_deserialize(rs, buf)?)))?),
    }.into())
}


