use anchor_lang::prelude::*;

declare_id!("EExeRoQMrfcJP28XQVjcE6khh3U8GC2RVZs28RNut5Br");

mod state;
mod placement;
mod global;
mod types;
mod board;
mod beastie_create;

pub use global::*;
pub use board::*;
pub use beastie_create::*;


#[program]
pub mod grid {
    use super::*;

    pub fn create_board(
        ctx: Context<CreateBoard>,
        seed: u64,
        owner: Pubkey,
        config: state::board::BoardConfig
    ) -> Result<()> {
        let board = &mut ctx.accounts.board;
        board.seed = seed;
        board.owner = owner;
        board.config = config;
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
}

