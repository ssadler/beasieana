use anchor_lang::prelude::*;


#[derive(Accounts)]
pub struct InitGlobal<'info> {
    #[account(
        init,
        space = 4096,
        payer = owner,
        seeds = [b"grid"],
        bump
    )]
    pub global: Account<'info, crate::state::global::Global>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}
