use anchor_lang::prelude::*;

declare_id!("D5VHC8DLUnuXCdpyrft9C1fiyshYmiJizzDsuGUwyfqK");

#[program]
pub mod signertest2 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("2: beastie is a signer {}", ctx.accounts.beastie.to_account_info().is_signer);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account()]
    pub beastie: Signer<'info>,
}
