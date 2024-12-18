use anchor_lang::prelude::*;

declare_id!("EExeRoQMrfcJP28XQVjcE6khh3U8GC2RVZs28RNut5Br");

mod remaining_accounts;
mod state;
mod placement;
mod global;
mod types;
mod board;
mod beastie_create;
mod admin;
mod billing;
mod links;

pub use global::*;
pub use board::*;
pub use beastie_create::*;
pub use admin::*;
pub use types::*;
pub use billing::*;
use placement::*;
use links::*;
use state::beastie::Link;


#[program]
pub mod grid {
    use state::beastie::Link;
    use types::CellPos;
    use beastie_common::{byte_ref, BOARD_KEY};

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

    pub fn init_beastie(ctx: Context<InitBeastie>, cell_id: u32) -> Result<()> {
        ctx.accounts.placement.cell_id = cell_id;
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
        mut ctx: Context<'_, '_, 'c, 'info, LinksContext<'info>>,
        links: Vec<Link>
    ) -> Result<()> where 'c: 'info {
        links::create_links(ctx, links)
    }

    pub fn remove_links<'c, 'info>(
        mut ctx: Context<'_, '_, 'c, 'info, LinksContext<'info>>,
        cells: Vec<u32>
    ) -> Result<()> where 'c: 'info {
        links::remove_links(ctx, cells)
    }

    //pub fn bill<'info>(mut ctx: Context<'_, '_, '_, 'info, BillContext>) -> Result<()> {
    //}

    //pub fn bill<'info>(mut ctx: Context<'_, '_, '_, 'info, PlacementContext<'info>>) -> Result<bool> {
    //    Ok(false)
    //    //let r = bill_beastie(&mut ctx)?;
    //    //if r == BillingResult::Broke {
    //    //    remove_beastie_from_board(ctx)?;
    //    //    Ok(false)
    //    //} else {
    //    //    Ok(true)
    //    //}
    //}
}

#[derive(Accounts)]
pub struct SayHello<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
}




