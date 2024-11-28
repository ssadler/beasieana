use anchor_lang::prelude::*;

declare_id!("8Gg4bD4regjmpvz2thxNkyjvPiyxUKTcLuLZpFh4XJpU");

static BEASTIE_KEY: &[u8] = b"asset.beastie";

macro_rules! byte_ref {
    ($val:expr, $size:expr) => {
        unsafe { &*(std::ptr::addr_of!($val) as *const [u8; $size]) }
    };
}


#[program]
pub mod beastie {
    use super::*;

    pub fn create_beastie(ctx: Context<CreateBeastie>, seed: u64, owner: Pubkey) -> Result<()> {

        let beastie = &mut ctx.accounts.beastie;
        beastie.seed = seed;
        beastie.creation_slot = Clock::get()?.slot;
        beastie.owner = owner;

        let seeds = [BEASTIE_KEY, byte_ref!(beastie.seed, 8)];
        let (pda, bump) = Pubkey::find_program_address(&seeds, ctx.program_id);
        if pda != beastie.key() {
            panic!("pda wrong");
        }

        beastie.bump = bump;

        // create beastie in grid

        let sseeds = [BEASTIE_KEY, byte_ref!(beastie.seed, 8), &[beastie.bump]];
        let signer_seeds = &[&sseeds[..]];
        let cpi = CpiContext::new_with_signer(
            ctx.accounts.grid_program.to_account_info(),
            grid::cpi::accounts::InitBeastie {
                asset_beastie: beastie.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                global: ctx.accounts.global.to_account_info(),
                grid_beastie: ctx.accounts.grid_beastie.to_account_info(),
            },
            signer_seeds
        );

        grid::cpi::init_beastie(cpi, seed)
    }

    pub fn send_token(ctx: Context<SendToken>, amount: u64) -> Result<()> {

        let beastie = &ctx.accounts.beastie;
        let seeds = [BEASTIE_KEY, byte_ref!(beastie.seed, 8), &[beastie.bump]];
        let signer_seeds = &[&seeds[..]];
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SplTransfer {
                from: ctx.accounts.beastie_ata.to_account_info(),
                to: ctx.accounts.dest_ata.to_account_info(),
                authority: ctx.accounts.beastie.to_account_info(),
            },
            signer_seeds
        );
        
        token::transfer(cpi_context, amount)
    }

    pub fn forward(ctx: Context<ForwardCall>) -> Result<()> {
        Ok(())
    }
}

use grid::program::Grid;


#[derive(Accounts)]
#[instruction(seed: u64, owner: Pubkey)]
pub struct CreateBeastie<'info> {
    #[account(
        init,
        payer = payer,
        space = 4096,
        seeds = [BEASTIE_KEY, seed.to_le_bytes().as_ref()],
        bump
    )]
    pub beastie: Account<'info, Beastie>,

    #[account(mut, seeds = [b"grid"], seeds::program = grid_program.key(), bump)]
    /// CHECK: yep
    pub global: UncheckedAccount<'info>,

    #[account(mut, seeds = [b"grid.beastie", seed.to_le_bytes().as_ref()], seeds::program = grid_program.key(), bump)]
    /// CHECK: yep
    pub grid_beastie: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub grid_program: Program<'info, Grid>,
}

#[account]
pub struct Beastie {
    pub bump: u8,
    pub seed: u64,
    pub creation_slot: u64,
    pub owner: Pubkey,
    pub placements: Vec<Pubkey>,
}


#[derive(Accounts)]
pub struct ForwardCall<'info> {
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

use anchor_spl::token::{self, Token, TokenAccount, Transfer as SplTransfer};



#[derive(Accounts)]
pub struct SendToken<'info> {
    #[account(
        seeds = [BEASTIE_KEY, byte_ref!(beastie.seed, 8)],
        bump,
        constraint = &beastie.owner == owner.key,
    )]
    pub beastie: Account<'info, Beastie>,

    #[account(mut)]
    pub beastie_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub dest_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,

    #[account()]
    pub owner: Signer<'info>,
}
