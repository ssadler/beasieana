use anchor_lang::prelude::*;

declare_id!("EExeRoQMrfcJP28XQVjcE6khh3U8GC2RVZs28RNut5Br");

mod state;
mod placement;
mod global;
mod types;
mod board;
mod beastie_create;
mod utils;
mod admin;
mod accounts_base;
mod billing;

pub use global::*;
pub use board::*;
pub use beastie_create::*;
pub use admin::*;
pub use placement::*;
pub use types::*;
pub use billing::*;


#[program]
pub mod grid {
    use types::CellPos;
    use utils::{byte_ref, BOARD_KEY};

    use super::*;

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

    pub fn init_beastie(
        ctx: Context<InitBeastie>,
        seed: u64
    ) -> Result<()> {

        let beastie = &mut ctx.accounts.grid_beastie;
        beastie.seed = seed;
        beastie.cell_id = ctx.accounts.global.next_cell_id();
        Ok(())
    }

    pub fn say(ctx: Context<SayHello>, sayit: String) -> Result<String> {
        Ok(sayit)
    }

    pub fn admin_init(ctx: Context<AdminInit>) -> Result<()> {
        ctx.accounts.global.admin = ctx.accounts.admin.key();
        Ok(())
    }
    pub fn admin_whitelist_token(ctx: Context<AdminWhitelistToken>, _token: Pubkey) -> Result<()> {
        ctx.accounts.grid_token_meta.enabled = true;
        Ok(())
    }

    pub fn place<'info>(ctx: Context<'_, '_, '_, 'info, PlacementContext<'info>>, pos: CellPos) -> Result<()> {
        place_beastie_on_board(ctx, pos)
    }

    pub fn remove<'info>(ctx: Context<'_, '_, '_, 'info, PlacementContext<'info>>) -> Result<()> {
        // TODO: Bill first
        remove_beastie_from_board(ctx)
    }

    pub fn bill<'info>(ctx: Context<'_, '_, '_, 'info, BillingContext<'info>>) -> Result<()> {
        bill_beastie(ctx)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SayHello<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
}
