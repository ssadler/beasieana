
use anchor_lang::{prelude::*, solana_program::sysvar::Sysvar};
use anchor_spl::token;
use crate::state::beastie::Placement;
use crate::{types::*, PlacementContext};

use beastie_common::{byte_ref, BOARD_KEY};


impl<'info> PlacementContext<'info> {
    fn commit_balance<'a, 'b>(&mut self, amount: u64) -> Result<()> {
        let k = self.board.key();
        self.cell.commitments.modify(k, |v| *v += amount);
        self.transfer_to_board(amount)
    }

    fn transfer_to_board(&self, amount: u64) -> Result<()> {
        let seeds = [BOARD_KEY, byte_ref!(self.board.seed, 8), &[self.board.bump]];
        let signer_seeds = [seeds.as_slice()];

        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.beastie_ata.to_account_info(),
                    to: self.board_ata.to_account_info(),
                    authority: self.beastie.to_account_info()
                },
                &signer_seeds
            ),
            amount
        )
    }

    pub fn bill_beastie(&mut self) -> Result<BillingResult> {

        let cell = self.cell.as_active();
        let height = Clock::get()?.slot;
        let diff = height - cell.billed_height;
        if diff == 0 {
            return Ok(BillingResult::Billed(0, 0));
        }

        let mut due = cell.rate * diff;


        // first take from committed
        let board_key = self.board.key();
        let committed = self.cell.commitments.get(&board_key);
        if committed > 0 {
            let take_c = std::cmp::min(due, committed);
            self.cell.commitments.modify(board_key, |v| *v -= take_c);
            due -= take_c;
        }

        // then take from ATA
        let bal = self.beastie_ata.amount;
        let take_t = std::cmp::min(bal, due);
        if take_t > 0 {
            due -= take_t;
            self.transfer_to_board(take_t)?;
        }

        let cell = self.cell.as_active_mut();
        cell.billed_height = height;

        if due > 0 {
            msg!("Beastie is broke; removing");
            cell.deactivate()?;
            return Ok(BillingResult::Broke);
        }

        Ok(BillingResult::Billed(0, 0))
    }


    pub fn start_billing<'c>(&mut self, pos: CellPos) -> Result<()> where 'c: 'info {

        let config = &self.board.config;

        // check min size, max size
        if pos.r < config.min_radius {
            panic!("min_radius");
        }
        if pos.r > config.max_radius {
            panic!("max_radius");
        }
        // check min value
        if self.beastie_ata.amount < config.add_cell_min_value {
            panic!("add_cell_min_value");
        }
        
        // Check placement is None
        if self.cell.is_active() {
            panic!("placement is active");
        }

        // Approve beastie for billing by board
        let approval = CpiContext::new(
            self.token_program.to_account_info(),
            token::Approve {
                to: self.beastie_ata.to_account_info(),
                delegate: self.board_ata.to_account_info(),
                authority: self.beastie.to_account_info()
            }
        );
        token::approve(approval, u64::MAX)?;

        let p = Placement {
            board: self.board.key(),
            billed_height: Clock::get()?.slot,
            rate: self.board.get_billing_rate(&pos),
            pos: pos.clone(),
            linked_balance: 0
        };
        self.cell.activate(p);

        Ok(())
    }
}


pub fn stop_billing<'c, 'info>(mut ctx: &PlacementContext<'info>) -> Result<()> where 'c: 'info {
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






#[derive(PartialEq, Eq)]
pub enum BillingResult {
    Billed(u64, u64),
    Broke
}

