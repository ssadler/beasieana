use anchor_lang::prelude::*;

use crate::state::global::*;


#[derive(Accounts)]
pub struct AdminInit<'info> {
    #[account(init, payer = payer, space = 4096, seeds = [b"grid"], bump)]
    pub global: Account<'info, Global>,

    #[account()]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}



#[derive(Accounts)]
#[instruction(token: Pubkey)]
pub struct AdminWhitelistToken<'info> {
    #[account(
        init,
        space = 1024,
        payer = payer,
        seeds = [b"token_meta", token.as_ref()],
        bump
    )]
    pub grid_token_meta: Account<'info, GridTokenMeta>,

    #[account(seeds = [b"grid"], bump)]
    pub global: Account<'info, Global>,
  
    #[account(constraint = admin.key() == global.admin)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

