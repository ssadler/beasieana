

use anchor_lang::prelude::*;

use crate::state::global::Global;

use account_macros::*;


//#[derive(Accounts)]
//#[instruction(token: Pubkey)]
//pub struct GlobalAccount<'info> {
//    #[account(init_if_needed, payer = payer, space = 4096, seeds = [b"grid"], bump)]
//    pub global: Account<'info, Global>,
//
//    #[account(mut)]
//    pub payer: Signer<'info>,
//    pub system_program: Program<'info, System>,
//}



//my_accounts!(
//    GlobalAccounts,
//    'info,
//    {
//        global!('info)
//    }
//);






pub mod macros {
    macro_rules! apply_attr {
        ($attr:ident, $item:item) => {
            #[::$attr]
            $item
        };
    }

    pub(crate) use apply_attr;    // <-- the trick
}

pub use macros::*;

