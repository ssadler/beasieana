use anchor_lang:: {
    prelude::*,
    solana_program::{
        program::invoke_signed,
        pubkey::Pubkey,
        instruction::Instruction,
    }
};
use crate::utils::*;
use beastie_common::Beastie;

#[derive(Accounts)]
pub struct ProxyCall<'info> {
    #[account(
        seeds = [BEASTIE_KEY, byte_ref!(beastie.seed, 8)],
        bump,
        constraint = &beastie.owner == owner.key
    )]
    pub beastie: Account<'info, Beastie>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,
}


#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct AccMeta {
    pubkey: Pubkey,
    is_signer: bool,
    is_writable: bool
}

impl Into<AccountMeta> for AccMeta {
    fn into(self) -> AccountMeta {
        AccountMeta {
            pubkey: self.pubkey,
            is_signer: self.is_signer,
            is_writable: self.is_writable
        }
    }
}




pub fn proxy(ctx: Context<ProxyCall>, data: Vec<u8>, accounts: Vec<AccMeta>) -> Result<()> {

    let seeds = [BEASTIE_KEY, byte_ref!(ctx.accounts.beastie.seed, 8), &[ctx.accounts.beastie.bump]];
    let signer_seeds = &[&seeds[..]];

    msg!("INVOKIIING");
    invoke_signed(
        &Instruction::new_with_bytes(
            ctx.remaining_accounts[0].key.clone(),
            data.as_slice(),
            // Can get this from remaining accounts too?
            accounts.into_iter().map(Into::into).collect()
        ),
        &ctx.remaining_accounts,
        signer_seeds
    )?;
    Ok(())
}
