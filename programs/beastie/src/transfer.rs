use anchor_lang::prelude::*;
use beastie_common::*;


#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(
        seeds = [BEASTIE_KEY, byte_ref!(beastie.cell_id, 4)],
        bump,
        constraint = &beastie.owner == owner.key
    )]
    pub beastie: Account<'info, Beastie>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,
}



pub fn transfer(ctx: Context<Transfer>, new_owner: Pubkey) -> Result<()> {
    ctx.accounts.beastie.owner = new_owner;
    Ok(())
}
