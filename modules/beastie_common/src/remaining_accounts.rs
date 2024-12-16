use std::{cell::RefCell, collections::HashMap};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address;
use crate::common::leak;
use spl_token::solana_program::{program::invoke_signed, system_instruction};



pub struct RemainingAccounts<'info> {
    rem: &'info [AccountInfo<'info>],
    cache: RefCell<(usize, HashMap<Pubkey, &'info AccountInfo<'info>>)>
}

impl<'info> RemainingAccounts<'info> {
    pub fn new(rem: &'info [AccountInfo<'info>]) -> RemainingAccounts<'info> {
        RemainingAccounts { rem, cache: (0, HashMap::new()).into() }
    }
    fn get(&self, addr: &Pubkey) -> AccountResult<'info> {
        if let Some(r) = self.cache.borrow().1.get(addr) {
            return Ok(*r);
        }

        let mut c = self.cache.borrow_mut();

        for pda in self.rem[c.0..].iter() {
            c.0 += 1;
            c.1.insert(pda.key.clone(), pda);
            if pda.key == addr {
                return Ok(pda);
            }
        }

        msg!("Could not find account: {}", addr);
        Err("Remaining accounts exhausted")
    }
    pub fn get_next(&self) -> Result<&'info AccountInfo<'info>> {
        let mut c = self.cache.borrow_mut();
        if let Some(account) = self.rem[c.0..].first() {
            c.0 += 1;
            c.1.insert(account.key.clone(), account);
            return Ok(account);
        }
        err!(Errors::NotEnoughAccounts)
    }
    pub fn get_ata(&self, owner: &Pubkey, token: &Pubkey) -> AccountResult<'info> {
        self.get(&get_associated_token_address(&owner, token))
    }
    pub fn get_pda(
        &self,
        mut seeds: Vec<&'static [u8]>,
        program_id: &'static Pubkey,
        init: InitPDA<'_, 'info>
    ) -> AccountResult<'info> {
        let (addr, bump_seed) = Pubkey::find_program_address(&seeds, program_id);
        let pda = self.get(&addr)?;
        if init.is_some() && pda.data_is_empty() {
            seeds.push(leak!([bump_seed]));
            let (payer, size) = init.unwrap();
            init_pda(pda, &seeds, program_id, payer, size).expect("failed to init pda");
        }
        Ok(pda)
    }

    pub fn load_pda<T: Clone + AccountSerialize + AccountDeserialize + Owner>(
        &self,
        program_id: &'static Pubkey,
        f: impl Fn(&Account<'info, T>) -> Vec<&'static [u8]>
    ) -> Result<Account<'info, T>> {
        let acc = Account::try_from(self.get_next()?)?;
        let seeds = f(&acc);
        let (addr, _) = Pubkey::find_program_address(&seeds, program_id);
        let info: &AccountInfo = acc.as_ref();
        require!(&addr == info.key, Errors::PDAAddressMismatch);
        Ok(acc)
    }
}

#[error_code]
pub enum Errors {
    #[msg("PDA address mismatch")]
    PDAAddressMismatch,

    #[msg("Not Enough Accounts")]
    NotEnoughAccounts,
}

pub type AccountResult<'info> = std::result::Result<&'info AccountInfo<'info>, &'static str>;


pub type InitPDA<'a, 'info> = Option<(&'a Signer<'info>, usize)>;





fn init_pda<'info>(
    account: &AccountInfo<'info>,
    seeds: &[&[u8]],
    program_id: &Pubkey,
    payer: &Signer<'info>, 
    size: usize
) -> Result<()> {

    if !account.is_writable { panic!("pda not writeable"); }

    invoke_signed(
        &system_instruction::create_account(
            payer.key,
            account.key,
            Rent::get()?.minimum_balance(size),
            size as u64,
            program_id
        ),
        &[payer.to_account_info(), account.clone()],
        &[seeds]
    )?;
    Ok(())
}



