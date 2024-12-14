use anchor_lang::prelude::*;
use beastie_common::*;

mod proxy;
mod transfer;

use proxy::*;
use transfer::*;

declare_id!("8Gg4bD4regjmpvz2thxNkyjvPiyxUKTcLuLZpFh4XJpU");



#[program]
pub mod beastie {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.global._next_cell_id += 1;
        Ok(())
    }

    pub fn create_beastie(ctx: Context<CreateBeastie>, owner: Pubkey) -> Result<()> {

        let beastie = &mut ctx.accounts.beastie;
        beastie.cell_id = ctx.accounts.global.next_cell_id();
        beastie.creation_slot = Clock::get()?.slot;
        beastie.owner = owner;

        let seeds = [BEASTIE_KEY, byte_ref!(beastie.cell_id, 4)];
        let (pda, bump) = Pubkey::find_program_address(&seeds, ctx.program_id);
        if pda != beastie.key() {
            panic!("pda wrong");
        }

        beastie.bump = bump;

        // create beastie in grid

        let sseeds = [BEASTIE_KEY, byte_ref!(beastie.cell_id, 4), &[ctx.bumps.beastie]];
        let signer_seeds = &[&sseeds[..]];
        let cpi = CpiContext::new_with_signer(
            ctx.accounts.grid_program.to_account_info(),
            grid::cpi::accounts::InitBeastie {
                beastie: beastie.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                placement: ctx.accounts.placement.to_account_info(),
            },
            signer_seeds
        );

        grid::cpi::init_beastie(cpi, beastie.cell_id)
    }

    pub fn proxy<'info>(
        ctx: Context<'_, '_, '_, 'info, ProxyCall<'info>>,
        data: Vec<u8>,
    ) -> Result<()> {
        crate::proxy::proxy(ctx, data)
    }

    pub fn transfer_ownership(ctx: Context<Transfer>, new_owner: Pubkey) -> Result<()> {
        crate::transfer::transfer(ctx, new_owner)
    }
}

use grid::program::Grid;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, space=1024, payer=payer, seeds = [b"global"], bump)]
    pub global: Account<'info, Global>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(owner: Pubkey)]
pub struct CreateBeastie<'info> {
    #[account(
        init,
        payer = payer,
        space = 4096,
        seeds = [BEASTIE_KEY, global._next_cell_id.to_le_bytes().as_ref()],
        bump
    )]
    pub beastie: Account<'info, Beastie>,

    #[account(mut, seeds = [b"global"], bump)]
    pub global: Account<'info, Global>,

    #[account(mut, seeds = [BEASTIE_PLACEMENT, global._next_cell_id.to_le_bytes().as_ref()], seeds::program = grid_program.key(), bump)]
    /// CHECK: yep
    pub placement: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub grid_program: Program<'info, Grid>,
}


#[account]
pub struct Global {
    _next_cell_id: u32,
}

impl Global {
    pub fn next_cell_id(&mut self) -> u32 {
        self._next_cell_id += 1;
        self._next_cell_id - 1
    }
}



// use anchor_spl::token::{self, Token, TokenAccount, Transfer as SplTransfer};
// 
// 
//     pub fn send_token(ctx: Context<SendToken>, amount: u64) -> Result<()> {
// 
//         let beastie = &ctx.accounts.beastie;
//         let seeds = [BEASTIE_KEY, byte_ref!(beastie.seed, 8), &[beastie.bump]];
//         let signer_seeds = &[&seeds[..]];
//         let cpi_context = CpiContext::new_with_signer(
//             ctx.accounts.token_program.to_account_info(),
//             SplTransfer {
//                 from: ctx.accounts.beastie_ata.to_account_info(),
//                 to: ctx.accounts.dest_ata.to_account_info(),
//                 authority: ctx.accounts.beastie.to_account_info(),
//             },
//             signer_seeds
//         );
//         
//         token::transfer(cpi_context, amount)
//     }
// 
// #[derive(Accounts)]
// pub struct SendToken<'info> {
//     #[account(
//         seeds = [BEASTIE_KEY, byte_ref!(beastie.seed, 8)],
//         bump,
//         constraint = &beastie.owner == owner.key,
//     )]
//     pub beastie: Account<'info, Beastie>,
// 
//     #[account(mut)]
//     pub beastie_ata: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub dest_ata: Account<'info, TokenAccount>,
//     pub token_program: Program<'info, Token>,
// 
//     #[account()]
//     pub owner: Signer<'info>,
// }
