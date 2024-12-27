use std::collections::BTreeMap;

use anchor_lang::prelude::*;

use crate::types::*;


pub trait ResourceDataSerialize: Sized {
    fn rd_serialize(&self) -> Vec<u8>;
}
impl<A: ResourceDataSerialize> ResourceDataSerialize for &A {
    fn rd_serialize(&self) -> Vec<u8> {
        (*self).rd_serialize()
    }
}

macro_rules! impl_serialize_int {
    ($type:tt) => {
        impl ResourceDataSerialize for $type {
            fn rd_serialize(&self) -> Vec<u8> {
                self.to_le_bytes().to_vec()
            }
        }
    };
}

impl_serialize_int!(u8);
impl_serialize_int!(u16);
impl_serialize_int!(u32);
impl_serialize_int!(u64);
impl_serialize_int!(u128);


macro_rules! impl_serialize_any {
    ($type:ty, |$sel:ident| $process:expr) => {
        impl ResourceDataSerialize for $type {
            fn rd_serialize(&$sel) -> Vec<u8> {
                $process
            }
        }
    };
}

impl_serialize_any!(bool, |self| (*self as u8).rd_serialize());
impl_serialize_any!(Pubkey, |self| self.clone().to_bytes().to_vec());
impl_serialize_any!(String, |self| Buffer(self.as_bytes().to_vec()).rd_serialize());

impl ResourceDataSerialize for Buffer {
    fn rd_serialize(&self) -> Vec<u8> {
        let mut out = (self.len() as u16).rd_serialize();
        out.extend(&self.0);
        out
    }
}

impl<A: ResourceDataSerialize, B: ResourceDataSerialize> ResourceDataSerialize for (A, B) {
    fn rd_serialize(&self) -> Vec<u8> {
        let mut out = self.0.rd_serialize();
        out.extend(self.1.rd_serialize());
        out
    }
}

impl<I: ResourceDataSerialize> ResourceDataSerialize for Option<I> {
    fn rd_serialize(&self) -> Vec<u8> {
        let mut out = self.is_some().rd_serialize();
        if let Some(o) = self {
            out.extend(o.rd_serialize());
        }
        out
    }
}
fn rd_iterator<T: ResourceDataSerialize, I: Iterator<Item=T>>(len: usize, iterator: I) -> Vec<u8> {
    let mut out = (len as u16).rd_serialize();
    iterator.for_each(|i| out.extend(i.rd_serialize()));
    out
}
impl<I: ResourceDataSerialize> ResourceDataSerialize for Vec<I> {
    fn rd_serialize(&self) -> Vec<u8> {
        rd_iterator(self.len(), self.iter())
    }
}
impl<I: ResourceDataSerialize> ResourceDataSerialize for BTreeMap<String, I> {
    fn rd_serialize(&self) -> Vec<u8> {
        rd_iterator(self.len(), self.iter())
    }
}



impl ResourceDataSerialize for RD {
    fn rd_serialize(&self) -> Vec<u8> {
        match self {
            RD::U8(u) => u.rd_serialize(),
            RD::U16(u) => u.rd_serialize(),
            RD::U32(u) => u.rd_serialize(),
            RD::U64(u) => u.rd_serialize(),
            RD::U128(u) => u.rd_serialize(),
            RD::Bool(b) => b.rd_serialize(),
            RD::String(s) => s.rd_serialize(),
            RD::Buffer(v) => v.rd_serialize(),
            RD::Pubkey(p) => p.rd_serialize(),
            RD::Option(o) => o.rd_serialize(),
            RD::List(v) => v.rd_serialize(),
            RD::Map(v) => v.rd_serialize(),
            RD::Struct(v) => v.rd_serialize()
        }
    }
}
