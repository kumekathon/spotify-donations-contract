use anchor_lang::prelude::*;

declare_id!("DTNMdyYqzgB7qu53RU6vVyhdERLAWvQKWZHnuAJ2d7k4");

#[program]
pub mod spotify_donations_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
