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

    pub fn create_beastie(ctx: Context<CreateBeastie>, cell_id: u32, owner: Pubkey) -> Result<()> {

        let beastie = &mut ctx.accounts.beastie;
        beastie.cell_id = cell_id;
        beastie.creation_slot = Clock::get()?.slot;
        beastie.owner = owner;

        // create beastie in grid

        let seeds = [BEASTIE_KEY, byte_ref!(cell_id, 4), &[ctx.bumps.beastie]];
        let signer_seeds = &[&seeds[..]];
        let cpi = CpiContext::new_with_signer(
            ctx.accounts.grid_program.to_account_info(),
            grid::cpi::accounts::InitBeastie {
                beastie: beastie.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                cell: ctx.accounts.cell.to_account_info(),
            },
            signer_seeds
        );

        grid::cpi::init_beastie(cpi, cell_id)
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
#[instruction(cell_id: u32, owner: Pubkey)]
pub struct CreateBeastie<'info> {
    #[account(
        init,
        payer = payer,
        space = 4096,
        seeds = [BEASTIE_KEY, cell_id.to_le_bytes().as_ref()],
        bump
    )]
    pub beastie: Account<'info, Beastie>,

    #[account(
        mut,
        seeds = [CELL_KEY, cell_id.to_le_bytes().as_ref()],
        seeds::program = grid_program.key(),
        bump
    )]
    /// CHECK: yep
    pub cell: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub grid_program: Program<'info, Grid>,
}

