use anchor_lang:: {
    prelude::*,
    solana_program::{
        program::invoke_signed,
        pubkey::Pubkey,
        instruction::Instruction,
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



pub fn proxy<'info>(ctx: Context<'_, '_, '_, 'info, ProxyCall<'info>>, data: Vec<u8>, prepends: u16) -> Result<()> {

    let seeds = [BEASTIE_KEY, byte_ref!(ctx.accounts.beastie.cell_id, 4), &[ctx.bumps.beastie]];
    let signer_seeds = &[&seeds[..]];
    let mut v = vec![];

    let remaining_accounts = if prepends == 0 {
        ctx.remaining_accounts
    } else {
        let l = (prepends & 0b111) as usize;
        for i in 0..l {
            v[i] = if prepends & (8<<i) == 0 {
                ctx.accounts.beastie.to_account_info()
            } else {
                ctx.accounts.owner.to_account_info()
            }
        }
        v.extend_from_slice(ctx.remaining_accounts);
        &v
    };

    invoke_signed(
        &Instruction::new_with_bytes(
            ctx.remaining_accounts[0].key.clone(),
            data.as_slice(),
            get_account_metas(&ctx)
        ),
        remaining_accounts,
        signer_seeds
    ).map_err(Into::into)
}

fn get_account_metas<'info>(ctx: &Context<'_, '_, '_, 'info, ProxyCall<'info>>) -> Vec<AccountMeta> {
    let b: &AccountInfo = ctx.accounts.beastie.as_ref();
    ctx.remaining_accounts[1..]
        .iter()
        .map(|a| {
            AccountMeta {
                pubkey: *a.key,
                is_signer: a.key == b.key || a.is_signer,
                is_writable: a.is_writable
            }
        })
        .collect::<Vec<_>>()
}
