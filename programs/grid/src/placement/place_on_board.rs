use anchor_lang::{prelude::*, solana_program::sysvar::Sysvar};
use anchor_spl::token;
use crate::{placement::quadtree::*, state::beastie::ActivePlacement};
use crate::types::*;
use crate::placement::context::*;



pub fn place_beastie_on_board<'info>(ctx: Context<'_, '_, '_, 'info, PlacementContext<'info>>, pos: CellPos) -> Result<()> {
    // check min size, max size
    if pos.r < ctx.accounts.board.config.min_radius {
        panic!("min_radius");
    }
    if pos.r > ctx.accounts.board.config.max_radius {
        panic!("max_radius");
    }
    // check min value
    if ctx.accounts.beastie_ata.amount < ctx.accounts.board.config.add_cell_min_value {
        panic!("add_cell_min_value");
    }
    // Check placement is None
    if ctx.accounts.placement.active.is_some() {
        panic!("placement is active");
    }

    // Approve beastie for billing by board
    let approval = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Approve {
            to: ctx.accounts.beastie_ata.to_account_info(),
            delegate: ctx.accounts.board_ata.to_account_info(),
            authority: ctx.accounts.asset_beastie.to_account_info()
        }
    );
    token::approve(approval, u64::MAX)?;

    ctx.accounts.grid_beastie.placement_board = Some(ctx.accounts.board.key());
    ctx.accounts.placement.active = Some(ActivePlacement {
        billed_height: Clock::get()?.slot,
        rate: ctx.accounts.board.get_billing_rate(&pos),
        pos: pos.clone()
    });

    place_beastie_on_grid(
        ctx.program_id,
        ctx.remaining_accounts,
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.grid_beastie.cell_id,
        ctx.accounts.board.key(),
        pos
    )
}

//fn placement_approval<'info>(accounts: &PlacementContext<'info>, amount: u64) -> Result<()> {
//    let beastie_ata = accounts.beastie_ata.to_account_info();
//    let board_ata = accounts.board_ata.to_account_info();
//    let beastie = accounts.asset_beastie.to_account_info();
//
//    let ix = spl_token::instruction::approve(
//        &spl_token::ID,
//        beastie_ata.key,
//        board_ata.key,
//        beastie.key,
//        &[],              // No signer pubkeys
//        amount,
//    )?;
//    anchor_lang::solana_program::program::invoke_signed(
//        &ix,
//        &[beastie_ata, board_ata, beastie],
//        &[]               // No signer seeds
//    ).map_err(Into::into)
//}

pub fn remove_beastie_from_board<'info>(ctx: Context<'_, '_, '_, 'info, PlacementContext<'info>>) -> Result<()> {
    let revocation = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Revoke {
            source: ctx.accounts.board_ata.to_account_info(),
            authority: ctx.accounts.asset_beastie.to_account_info()
        }
    );
    token::revoke(revocation)?;

    let placement = ctx.accounts.placement.active.take().expect("no placement?");
    ctx.accounts.placement.active = None;

    remove_beastie_from_grid(
        ctx.program_id,
        ctx.remaining_accounts,
        ctx.accounts.grid_beastie.cell_id,
        ctx.accounts.board.key(),
        placement.pos
    )
}
