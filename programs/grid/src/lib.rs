use anchor_lang::prelude::*;

declare_id!("EExeRoQMrfcJP28XQVjcE6khh3U8GC2RVZs28RNut5Br");

mod state;
mod placement;
mod global;
mod types;
mod board;
mod admin;
mod billing;
mod links;
mod remaining_accounts;

use beastie_common::*;
pub use global::*;
pub use board::*;
pub use admin::*;
pub use types::*;
pub use billing::*;
use placement::*;
use state::beastie::{Cell, Link};


#[program]
pub mod grid {
    use remaining_accounts::CTX;
    use state::beastie::Link;
    use types::CellPos;
    use beastie_common::{byte_ref, BOARD_KEY};

    use super::*;

    pub fn init_cell<'info>(ctx: Context<InitCellContext<'info>>) -> Result<()> {
        ctx.accounts.cell.cell_id = ctx.accounts.beastie.cell_id;
        Ok(())
    }

    pub fn init_placement(ctx: Context<InitPlacementContext>) -> Result<()> {
        ctx.accounts.cell.cell_id = ctx.accounts.beastie.cell_id;
        Ok(())
    }

    pub fn create_board(
        ctx: Context<CreateBoard>,
        seed: u64,
        owner: Pubkey,
        token: Pubkey,
        config: state::board::BoardConfig
    ) -> Result<()> {
        let board = &mut ctx.accounts.board;
        board.seed = seed;
        board.owner = owner;
        board.token = token;
        board.config = config;

        let seeds = [BOARD_KEY, byte_ref!(seed, 8)];
        let (pda, bump) = Pubkey::find_program_address(&seeds, ctx.program_id);
        if pda != board.key() {
            panic!("pda wrong");
        }
        board.bump = bump;

        Ok(())
    }

    pub fn admin_init(ctx: Context<AdminInit>) -> Result<()> {
        ctx.accounts.global.admin = ctx.accounts.admin.key();
        Ok(())
    }
    pub fn admin_whitelist_token(ctx: Context<AdminWhitelistToken>, _token: Pubkey) -> Result<()> {
        ctx.accounts.grid_token_meta.enabled = true;
        Ok(())
    }

    pub fn place<'c, 'info>(ctx: Context<'_, '_, 'c, 'info, PlacementContext<'info>>, pos: CellPos) -> Result<()> where 'c: 'info {
        placement::place(ctx, pos)
    }

    pub fn remove<'c, 'info>(ctx: Context<'_, '_, 'c, 'info, PlacementContext<'info>>) -> Result<()> where 'c: 'info {
        placement::remove(ctx)
    }

    pub fn create_links<'c, 'info>(
        ctx: Context<'_, '_, 'c, 'info, PlacementContext<'info>>,
        links: Vec<Link>
    ) -> Result<()> where 'c: 'info {
        let mut ctx = CTX::new(ctx);
        for link in links {
            links::create_link(&mut ctx, link)?;
        }
        Ok(())
    }

    pub fn remove_links<'c, 'info>(
        ctx: Context<'_, '_, 'c, 'info, PlacementContext<'info>>,
        cells: Vec<u32>
    ) -> Result<()> where 'c: 'info {
        let mut ctx = CTX::new(ctx);
        for cell in cells {
            links::remove_link(&mut ctx, cell)?;
        }
        Ok(())
    }

    pub fn bill_me<'c, 'info>(
        ctx: Context<'_, '_, '_, 'info, PlacementContext<'info>>,
    ) -> Result<bool> {
        Ok(true)
    }

    pub fn check_me<'c, 'info>(
        ctx: Context<'_, '_, '_, 'info, PlacementContext<'info>>,
    ) -> Result<()> {
        Ok(())
    }

    pub fn beastie_is_active<'info>(
        ctx: Context<'_, '_, '_, 'info, BeastieIsActive<'info>>
    ) -> Result<bool> {
        Ok(ctx.accounts.cell.is_active())
    }

    pub fn noop<'info>(_ctx: Context<NoopContext<'info>>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct NoopContext<'info> {
    #[account(
        signer,
        seeds = [BEASTIE_KEY, byte_ref!(beastie.cell_id, 4)],
        seeds::program = BEASTIE_PROGRAM_ID,
        bump
    )]
    pub beastie: Box<Account<'info, Beastie>>,
}

#[derive(Accounts)]
pub struct InitCellContext<'info> {
    #[account(
        signer,
        seeds = [BEASTIE_KEY, byte_ref!(beastie.cell_id, 4)],
        seeds::program = BEASTIE_PROGRAM_ID,
        bump
    )]
    pub beastie: Account<'info, Beastie>,
    #[account(
        init,
        payer = payer,
        space = 4096,
        seeds = [CELL_KEY, byte_ref!(beastie.cell_id, 4)],
        bump
    )]
    pub cell: Account<'info, Cell>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct BeastieIsActive<'info> {
    #[account(
        seeds = [BEASTIE_KEY, byte_ref!(beastie.cell_id, 4)],
        seeds::program = BEASTIE_PROGRAM_ID,
        bump
    )]
    pub beastie: Account<'info, Beastie>,
    #[account(
        seeds = [CELL_KEY, byte_ref!(beastie.cell_id, 4)],
        bump
    )]
    pub cell: Account<'info, Cell>
}

