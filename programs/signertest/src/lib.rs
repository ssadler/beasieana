use anchor_lang::prelude::*;

use beastie_common::*;
use signertest2::program::Signertest2;

declare_id!("244yWpGi8ba5Yni1wvdgYhWzRvkFpCtfvShN7JF9zn53");

#[program]
pub mod signertest {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let s = ctx.accounts.beastie.to_account_info().is_signer;
        msg!("1: beastie is a signer {}", s);
        //if s {
        //    let cpi = CpiContext::new(
        //        ctx.accounts.signertest2_program.to_account_info(),
        //        signertest2::cpi::accounts::Initialize {
        //            beastie: ctx.accounts.beastie.to_account_info()
        //        }
        //    );
        //    signertest2::cpi::initialize(cpi)?;
        //}
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account()]
    pub beastie: Signer<'info>,
    pub signertest2_program: Program<'info, Signertest2>,
}
