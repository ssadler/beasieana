use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use anchor_lang::prelude::*;
use anchor_spl::token;
use beastie_common::{byte_ref, BEASTIE_KEY, BEASTIE_PROGRAM_ID};

use crate::CellPos;
use crate::CellPositionedId;


#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct Active;
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct MaybeActive;

pub trait BeastieCellState {}
impl BeastieCellState for Active {}
impl BeastieCellState for MaybeActive {}

#[account]
pub struct Cell<T: BeastieCellState = MaybeActive> {
    pub cell_id: u32,
    active: Option<Placement>, // not a pub field
    pub incoming_links: u16,
    pub commitments: Commitments,
    pub links: Vec<EffectiveLink>,
    _state: PhantomData<T>
}

pub type ActiveCell = Cell<Active>;

impl<T: BeastieCellState> Cell<T> {
    pub fn asset_address(&self) -> Pubkey {
        let seeds = [BEASTIE_KEY, byte_ref!(self.cell_id, 4)];
        let (addr, _) = Pubkey::find_program_address(&seeds, &BEASTIE_PROGRAM_ID);
        addr
    }
    pub fn unapply_link(&mut self, link: &Link, effectiveness: u8) {
        if let Some(p) = self.active.as_mut() {
            p.linked_balance -= link.get_effect(effectiveness);
        }
        self.incoming_links -= 1;
    }
}

impl Cell<MaybeActive> {
    pub fn as_active(&self) -> &ActiveCell {
        assert!(self.active.is_some(), "as_active: not active");
        unsafe { &*(self as *const Cell as *const ActiveCell) }
    }
    pub fn as_active_mut(&mut self) -> &mut ActiveCell {
        assert!(self.active.is_some(), "as_active: not active");
        unsafe { &mut *(self as *mut Cell as *mut ActiveCell) }
    }
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }
    pub fn activate(&mut self, p: Placement) {
        assert!(self.active.is_none(), "activate: already active");
        assert!(self.links.len() == 0, "activate: have links");
        assert!(self.incoming_links == 0, "activate: have incoming links");
        self.active.replace(p);
    }
}

impl ActiveCell {
    pub fn apply_link(&mut self, link: &Link, effectiveness: u8) {
        self.linked_balance += link.get_effect(effectiveness);
        self.incoming_links += 1;
    }
    pub fn get_cell(&self) -> CellPositionedId {
        CellPositionedId { cell_id: self.cell_id, pos: (*self).pos }
    }
    pub fn deactivate(&mut self) -> Result<Placement> {
        let h = Clock::get()?.slot;
        assert!(self.billed_height == h, "deactivate: not billed");
        Ok(self.active.take().unwrap())
    }
}


impl Deref for ActiveCell {
    type Target = Placement;
    fn deref(&self) -> &Self::Target {
        self.active.as_ref().unwrap()
    }
}
impl DerefMut for ActiveCell {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.active.as_mut().unwrap()
    }
}



#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct Placement {
    pub board: Pubkey,
    pub pos: CellPos,
    pub billed_height: u64,
    pub rate: u64,
    pub linked_balance: i64,
}

impl Placement {
    pub fn get_due(&self) -> Result<u64> {
        let height = Clock::get()?.slot;
        Ok((height - self.billed_height) * self.rate)
    }
}


// Link effectiveness is (distance / config.link_max_distance * 255)
pub type EffectiveLink = (Link, u8);


#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub struct Link {
    pub cell_id: u32,
    pub amount: u64,
    pub typ: LinkType,
}

impl Link {
    pub fn get_effect(&self, effectiveness: u8) -> i64 {
        ((self.amount * effectiveness as u64) as i64 / 255) * self.typ.multiplier()
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum LinkType {
    Positive,
    Negative
}

impl LinkType {
    pub fn multiplier(&self) -> i64 {
        if *self == LinkType::Positive { 1 } else { -1 }
    }
}


pub trait HasActiveBeastie {
    fn get_cell(&self) -> &ActiveCell;
    fn get_ata(&self) -> &token::TokenAccount;
    fn beastie_free_balance(&self) -> u64 {
        self.get_ata().amount - self.get_cell().links.iter().map(|l| l.0.amount).sum::<u64>()
    }
    fn beastie_security_balance(&self) -> Result<i64> {
        let cell = self.get_cell();
        let c = cell.commitments.get(&cell.board);
        Ok(
            (c + self.beastie_free_balance() - cell.get_due()?) as i64
            + cell.linked_balance
        )
    }
}

impl HasActiveBeastie for (&ActiveCell, &token::TokenAccount) {
    fn get_cell(&self) -> &ActiveCell {
        self.0
    }
    fn get_ata(&self) -> &token::TokenAccount {
        self.1
    }
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct Commitments(pub Vec<Commitment>);
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct Commitment {
    pub key: Pubkey,
    pub amount: u64
}


impl Commitments {
    pub fn get<'a>(&'a self, key: &Pubkey) -> u64 {
        for item in self.0.iter() {
            if &item.key == key {
                return item.amount;
            }
        }
        0
    }
    pub fn modify<F: FnOnce(&mut u64)>(&mut self, key: Pubkey, f: F) {
        if let Some(idx) = self.0.iter().position(|c| c.key == key) {
            f(&mut self.0[idx].amount);
            if self.0[idx].amount == 0 {
                self.0.remove(idx);
            }
        } else {
            let mut amount = 0;
            f(&mut amount);
            if amount > 0 {
                self.0.push(Commitment { key, amount });
            }
        }
    }
}
