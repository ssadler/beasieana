use std::{cell::RefCell, collections::HashMap, ops::{Deref, DerefMut}};
use anchor_lang::{prelude::*, Bumps};
use anchor_spl::associated_token::get_associated_token_address;
use beastie_common::{leak, BEASTIE_PLACEMENT, GRID_PROGRAM_ID, PAD_KEY};
use spl_token::solana_program::{program::invoke_signed, system_instruction};



pub struct CTX<'a, 'b, 'c, 'info, A: Bumps> {
    pub ctx: Context<'a, 'b, 'c, 'info, A>,
    pub rem: RemainingAccounts<'info>
}

impl<'a, 'b, 'c, 'info, A: Bumps> Deref for CTX<'a, 'b, 'c, 'info, A> {
    type Target = Context<'a, 'b, 'c, 'info, A>;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}
impl<'a, 'b, 'c, 'info, A: Bumps> DerefMut for CTX<'a, 'b, 'c, 'info, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

impl<'a, 'b, 'c, 'info, A: Bumps> CTX<'a, 'b, 'c, 'info, A> {
    pub fn new(ctx: Context<'a, 'b, 'c, 'info, A>)
        -> CTX<'a, 'b, 'c, 'info, A> where 'c: 'info
    {
        let r = ctx.remaining_accounts;
        CTX { ctx, rem: RemainingAccounts::new(r) }
    }
}





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
    pub fn get_ata(&self, owner: &Pubkey, token: &Pubkey) -> AccountResult<'info> {
        self.get(&get_associated_token_address(&owner, token))
    }
    pub fn get_pda(&self, mut seeds: Vec<&'static [u8]>, program_id: &'static Pubkey, init: InitPDA<'_, 'info>) -> AccountResult<'info> {
        let (addr, bump_seed) = Pubkey::find_program_address(&seeds, program_id);
        let pda = self.get(&addr)?;
        if let Some((payer, size)) = init {
            seeds.push(leak!([bump_seed]));
            init_pda(pda, &seeds, program_id, payer, size);
        }
        Ok(pda)
    }
    pub fn get_placement(&self, cell_id: u32, init: InitPDA<'_, 'info>) -> AccountResult<'info> {
        let other_beastie_seeds = vec![BEASTIE_PLACEMENT, leak!(cell_id.to_le_bytes())];
        self.get_pda(other_beastie_seeds, &GRID_PROGRAM_ID, init)
    }
    pub fn get_pad(&self, space_key: &'static [u8], xx: u16, yy: u16, init: InitPDA<'_, 'info>) -> AccountResult<'info> {
        let seeds = vec![PAD_KEY, space_key, leak!(xx.to_le_bytes()), leak!(yy.to_le_bytes())];
        self.get_pda(seeds, &GRID_PROGRAM_ID, init)
    }
}

type AccountResult<'info> = std::result::Result<&'info AccountInfo<'info>, &'static str>;


pub type InitPDA<'a, 'info> = Option<(&'a Signer<'info>, usize)>;





 fn init_pda<'info>(
    account: &AccountInfo<'info>,
    seeds: &[&[u8]],
    program_id: &Pubkey,
    payer: &Signer<'info>, 
    size: usize
) -> Result<()> {
    if !account.is_writable { panic!("pda not writeable"); }

    if account.data_is_empty() {
        let reb = Rent::get()?.minimum_balance(size);

        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                account.key,
                reb,
                size as u64,
                program_id
            ),
            &[payer.to_account_info(), account.clone()],
            &[seeds]
        )?;
    }
    Ok(())
}



