
use anchor_lang::{prelude::*, solana_program::sysvar::Sysvar};
use anchor_spl::token;
use crate::state::beastie::Placement;
use crate::types::*;

use anchor_spl::token::TokenAccount;
use beastie_common::{byte_ref, Beastie, BOARD_KEY};

use crate::state::{beastie::Cell, board::Board};



pub trait BillingContext<'info> : Sized {
    fn board_ata(&self) -> &Account<'info, TokenAccount>;
    fn billing_board(&self) -> &Account<'info, Board>;
    fn billing_token_program(&self) -> AccountInfo<'info>;
    fn get_beastie(&self) -> &Account<'info, Beastie>;
    fn get_placement(&mut self) -> &mut Account<'info, Cell>;
    fn beastie_ata(&self) -> &Account<'info, token::TokenAccount>;
    fn commit_balance<'a, 'b>(&mut self, amount: u64) -> Result<()> {
        let k = self.billing_board().key();
        self.get_placement().commitments.modify(k, |v| *v += amount);
        self.transfer_to_board(amount)
    }
    fn transfer_to_board<'a, 'b>(&'a self, amount: u64) -> Result<()> {
        let board = self.billing_board();
        let seeds = [BOARD_KEY, byte_ref!(board.seed, 8), &[board.bump]];
        let signer_seeds = [seeds.as_slice()];

        token::transfer(
            CpiContext::new_with_signer(
                self.billing_token_program(),
                token::Transfer {
                    from: self.beastie_ata().to_account_info(),
                    to: self.board_ata().to_account_info(),
                    authority: self.get_beastie().to_account_info()
                },
                &signer_seeds
            ),
            amount
        )
    }
}

#[derive(PartialEq, Eq)]
pub enum BillingResult {
    Billed(u64, u64),
    Broke
}

pub fn bill_beastie<'info, C: BillingContext<'info>>(ctx: &mut C) -> Result<BillingResult> {

    let cell = ctx.get_placement().as_active();
    let height = Clock::get()?.slot;
    let diff = height - cell.billed_height;
    if diff == 0 {
        return Ok(BillingResult::Billed(0, 0));
    }

    let mut due = cell.rate * diff;


    // first take from committed
    let board_key = ctx.billing_board().key();
    let committed = ctx.get_placement().commitments.get(&board_key);
    if committed > 0 {
        let take_c = std::cmp::min(due, committed);
        ctx.get_placement().commitments.modify(board_key, |v| *v -= take_c);
        due -= take_c;
    }

    // then take from ATA
    let bal = ctx.beastie_ata().amount;
    let take_t = std::cmp::min(bal, due);
    if take_t > 0 {
        due -= take_t;
        ctx.transfer_to_board(take_t)?;
    }

    let cell = ctx.get_placement().as_active_mut();
    cell.billed_height = height;

    if due > 0 {
        msg!("Beastie is broke; removing");
        cell.deactivate()?;
        return Ok(BillingResult::Broke);
    }

    Ok(BillingResult::Billed(0, 0))
}





pub fn start_billing<'c, 'info, C: BillingContext<'info>>(ctx: &mut C, pos: CellPos) -> Result<()> where 'c: 'info {

    let config = &ctx.billing_board().config;

    // check min size, max size
    if pos.r < config.min_radius {
        panic!("min_radius");
    }
    if pos.r > config.max_radius {
        panic!("max_radius");
    }
    // check min value
    if ctx.beastie_ata().amount < config.add_cell_min_value {
        panic!("add_cell_min_value");
    }
    
    // Check placement is None
    if ctx.get_placement().is_active() {
        panic!("placement is active");
    }

    // Approve beastie for billing by board
    let approval = CpiContext::new(
        ctx.billing_token_program().to_account_info(),
        token::Approve {
            to: ctx.beastie_ata().to_account_info(),
            delegate: ctx.board_ata().to_account_info(),
            authority: ctx.get_beastie().to_account_info()
        }
    );
    token::approve(approval, u64::MAX)?;

    let p = Placement {
        board: ctx.billing_board().key(),
        billed_height: Clock::get()?.slot,
        rate: ctx.billing_board().get_billing_rate(&pos),
        pos: pos.clone(),
        linked_balance: 0
    };
    ctx.get_placement().activate(p);

    Ok(())
}


pub fn stop_billing<'c, 'info, C: BillingContext<'info>>(mut ctx: &C) -> Result<()> where 'c: 'info {
    Ok(())
    // TODO   (the below Revoke should be correct)

    //let revocation = CpiContext::new(
    //    ctx.billing_token_program(),
    //    token::Revoke {
    //        source: ctx.beastie_ata().to_account_info(),
    //        authority: ctx.get_beastie().to_account_info()
    //    }
    //);
    //token::revoke(revocation)?;

    //let placement = ctx.accounts.placement.active.take().expect("no placement?");
}





