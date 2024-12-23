use anchor_lang::prelude::*;
use beastie_common::*;

mod proxy;
mod transfer;

use grid::program::Grid;
use proxy::*;
use transfer::*;

declare_id!("8Gg4bD4regjmpvz2thxNkyjvPiyxUKTcLuLZpFh4XJpU");



#[program]
pub mod beastie {

    use super::*;

    pub fn create_beastie(ctx: Context<CreateBeastieContext>, cell_id: u32, owner: Pubkey) -> Result<()> {
        let beastie = &mut ctx.accounts.beastie;
        beastie.creation_time = Clock::get()?.unix_timestamp;
        beastie.cell_id = cell_id;
        beastie.owner = owner;
        beastie.exit(ctx.program_id)?; // Write

        // call to grid to init cell
        let signer = &[BEASTIE_KEY, byte_ref!(cell_id, 4), &[ctx.bumps.beastie]];
        let ssigner: &[&[&[u8]]] = &[signer];
        let cpi = CpiContext::new_with_signer(
            ctx.accounts.grid_program.to_account_info(),
            grid::cpi::accounts::InitCellContext {
                beastie: beastie.to_account_info(),
                cell: ctx.accounts.cell.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info()
            },
            ssigner
        );
        grid::cpi::init_cell(cpi)
    }

    pub fn proxy<'info>(
        ctx: Context<'_, '_, '_, 'info, MultiProxyContext<'info>>,
        data: Vec<ProxyCall>,
    ) -> Result<()> {
        crate::proxy::multi_proxy(ctx, data)
    }

    pub fn transfer_ownership(ctx: Context<BeastieOwnerAction>, new_owner: Pubkey) -> Result<()> {
        ctx.accounts.beastie.owner = new_owner;
        Ok(())
    }

    pub fn give_notice(ctx: Context<BeastieOwnerAction>) -> Result<()> {
        assert!(
            ctx.accounts.beastie.notice_given_time.replace(Clock::get()?.unix_timestamp).is_none(),
            "notice already given"
        );
        Ok(())
    }

    pub fn system_override(ctx: Context<BeastieOwnerAction>, action: OverrideAction) -> Result<()> {

        #[cfg(feature = "production")]
        panic!("system override denied");

        match action {
            OverrideAction::SetNoticeFulfilled => {
                let t = Clock::get()?.unix_timestamp;
                ctx.accounts.beastie.notice_given_time.replace(t-100000);
                ctx.accounts.beastie.exit(ctx.program_id)?;
            }
        }

        Ok(())
    }

    pub fn reset_notice(ctx: Context<BeastieOwnerAction>) -> Result<()> {
        ctx.accounts.beastie.notice_given_time = None;
        Ok(())
    }

    pub fn noop(ctx: Context<NoopContext>) -> Result<()> {
        Ok(())
    }
}


#[derive(Accounts)]
pub struct NoopContext {}

#[derive(Accounts)]
#[instruction(cell_id: u32, owner: Pubkey)]
pub struct CreateBeastieContext<'info> {
    #[account(
        init,
        payer = payer,
        space = 4096,
        seeds = [BEASTIE_KEY, cell_id.to_le_bytes().as_ref()],
        bump
    )]
    pub beastie: Account<'info, Beastie>,

    #[account(mut)]
    /// CHECK: Will be checked by Grid
    pub cell: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub grid_program: Program<'info, Grid>,
    pub system_program: Program<'info, System>,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum OverrideAction {
    SetNoticeFulfilled
}

