use anchor_lang:: {
    prelude::*,
    solana_program::{
        instruction::Instruction, program::invoke_signed_unchecked, pubkey::Pubkey
    }
};
use beastie_common::*;

#[derive(Accounts)]
pub struct ProxyCall<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [BEASTIE_KEY, byte_ref!(beastie.cell_id, 4)],
        bump,
        constraint = &beastie.owner == owner.key
    )]
    pub beastie: Account<'info, Beastie>,
}



pub fn proxy<'info>(ctx: Context<'_, '_, '_, 'info, ProxyCall<'info>>, data: Vec<u8>) -> Result<()> {

    let seeds = [BEASTIE_KEY, byte_ref!(ctx.accounts.beastie.cell_id, 4), &[ctx.bumps.beastie]];
    let signer_seeds = &[&seeds[..]];

    invoke_signed_unchecked(
        &Instruction::new_with_bytes(
            ctx.remaining_accounts[0].key.clone(),
            data.as_slice(),
            get_account_metas(&ctx)
        ),
        &ctx.remaining_accounts[1..],
        signer_seeds
    ).map_err(Into::into)
}

fn get_account_metas<'info>(ctx: &Context<'_, '_, '_, 'info, ProxyCall<'info>>) -> Vec<AccountMeta> {
    let b: &AccountInfo = ctx.accounts.beastie.as_ref();

    ctx.remaining_accounts[1..]
        .iter()
        .map(|a|
            AccountMeta {
                pubkey: *a.key,
                is_signer: a.key == b.key || a.is_signer,
                is_writable: a.is_writable
            }
        )
        .collect::<Vec<_>>()
}
