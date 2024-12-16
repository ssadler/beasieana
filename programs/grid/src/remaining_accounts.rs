use anchor_lang::{prelude::*, Bumps};
use beastie_common::{impl_deref, impl_deref_const, leak, remaining_accounts as RA, CELL_KEY, GRID_PROGRAM_ID, PAD_KEY};

pub use RA::InitPDA;



pub struct CTX<'a, 'b, 'c, 'info, A: Bumps> {
    pub ctx: Context<'a, 'b, 'c, 'info, A>,
    pub rem: RemainingAccounts<'info>
}
impl_deref!(['a, 'b, 'c, 'info, A: Bumps], CTX<'a, 'b, 'c, 'info, A>, Context<'a, 'b, 'c, 'info, A>, ctx);
impl<'a, 'b, 'c, 'info, A: Bumps> CTX<'a, 'b, 'c, 'info, A> {
    pub fn new(ctx: Context<'a, 'b, 'c, 'info, A>)
        -> CTX<'a, 'b, 'c, 'info, A> where 'c: 'info
    {
        let r = ctx.remaining_accounts;
        CTX { ctx, rem: RemainingAccounts(RA::RemainingAccounts::new(r)) }
    }
}





pub struct RemainingAccounts<'info>(pub RA::RemainingAccounts<'info>);
impl_deref_const!(['info], RemainingAccounts<'info>, RA::RemainingAccounts<'info>, 0);


impl<'info> RemainingAccounts<'info> {
    pub fn get_cell(&self, cell_id: u32, init: InitPDA<'_, 'info>) -> RA::AccountResult<'info> {
        let other_beastie_seeds = vec![CELL_KEY, leak!(cell_id.to_le_bytes())];
        self.get_pda(other_beastie_seeds, &GRID_PROGRAM_ID, init)
    }
    pub fn get_pad(&self, space_key: &'static [u8], xx: u16, yy: u16, init: InitPDA<'_, 'info>) -> RA::AccountResult<'info> {
        let seeds = vec![PAD_KEY, space_key, leak!(xx.to_le_bytes()), leak!(yy.to_le_bytes())];
        self.get_pda(seeds, &GRID_PROGRAM_ID, init)
    }
}
