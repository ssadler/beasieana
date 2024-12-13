use anchor_lang::prelude::*;
use std::ops::{Deref, DerefMut};



#[derive(Clone)]
pub struct InteriorMutability<T>(pub std::cell::RefCell<T>);

impl<T: AnchorSerialize> AnchorSerialize for InteriorMutability<T> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.0.borrow().serialize(writer)
    }
}
impl<T: AnchorDeserialize> AnchorDeserialize for InteriorMutability<T> {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        T::deserialize_reader(reader).map(std::cell::RefCell::new).map(InteriorMutability)
    }
}

impl<T> Deref for InteriorMutability<T> {
    type Target = std::cell::RefCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}



#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct Commitment {
    pub key: Pubkey,
    pub amount: u64
}

pub fn defaultmap_get<'a>(dm: &'a Vec<Commitment>, key: &Pubkey) -> u64 {
    for item in dm.iter() {
        if &item.key == key {
            return item.amount;
        }
    }
    0
}
pub fn defaultmap_modify<F: FnOnce(&mut u64)>(dm: &mut Vec<Commitment>, key: Pubkey, f: F) {
    if let Some(idx) = dm.iter().position(|c| c.key == key) {
        f(&mut dm[idx].amount);
        if dm[idx].amount == 0 {
            dm.remove(idx);
        }
    } else {
        let mut amount = 0;
        f(&mut amount);
        if amount > 0 {
            dm.push(Commitment { key, amount });
        }
    }
}
