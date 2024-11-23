use anchor_lang::prelude::*;

declare_id!("4M9GhRRrJAwDZph7ybJWLEYT1tN5xY3MTiW3QtRFVBVD");

#[program]
pub mod beastieana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    //pub fn place(ctx: Context<
}

#[derive(Accounts)]
pub struct Initialize {}
