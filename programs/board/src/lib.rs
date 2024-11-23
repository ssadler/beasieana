use anchor_lang::prelude::*;

use state::{board::*, Grid};
use placement::*;

pub mod state;
pub mod types;
pub mod placement;

declare_id!("EGphNMLtdsM4EkuhY1XkhUBfSq1bDShHp5ejpdEv5s8n");

#[program]
pub mod board {
    use anchor_lang::solana_program::log::sol_log;
    use types::CellPos;

    use super::*;

    pub fn create_grid(ctx: Context<CreateGrid>) -> Result<()> {
        Ok(())
    }

    pub fn create_board(ctx: Context<CreateBoard>, config: BoardConfig) -> Result<()> {
        ctx.accounts.board.create(config)
    }

    pub fn place<'info>(ctx: Context<'_, '_, '_, 'info, PlaceBeastie<'info>>, x: u16, y: u16, r: u16) -> Result<()> {
        place_beastie(ctx, CellPos { x, y, r })
    }
}


#[derive(Accounts)]
pub struct CreateBoard<'info> {
    #[account(init, payer = owner, space = 4096)]
    pub board: Account<'info, Board>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateGrid<'info> {
    #[account(init, payer = owner, space = 4096)]
    pub grid: Account<'info, Grid>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}
