use anchor_lang::prelude::*;
use anchor_spl::token::Token;

declare_id!("DTNMdyYqzgB7qu53RU6vVyhdERLAWvQKWZHnuAJ2d7k4");

const COMMISSION: f64 = 0.01;

#[account]
pub struct State {
    pub account_donate_balances: Vec<(String, u64)>, // Replace BTreeMap with Vec
    pub account_withdraw_balances: Vec<(String, u64)>, // Replace BTreeMap with Vec
    pub owner: Pubkey,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 32 + 1024)]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Donate<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub from: Signer<'info>,
    /// CHECK: This is safe because we manually verify it in our logic
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DistributeTokens<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub from: Signer<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub from: Signer<'info>,
    /// CHECK: This is safe because we manually verify it in our logic
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct DistributionTokens {
    pub account_id: String,
    pub distribution: Vec<(String, f32)>, // Replace HashMap with Vec
}

#[error_code]
pub enum ErrorCode {
    #[msg("MustBeOwner")]
    MustBeOwner,
}

#[program]
pub mod spotify_donations_contract {
    use anchor_spl::token::{self, Transfer};
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.account_donate_balances = Vec::new();
        state.account_withdraw_balances = Vec::new();
        state.owner = *ctx.accounts.user.key;

        Ok(())
    }

    pub fn donate(ctx: Context<Donate>, account_id: String, amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;

        if let Some(balance) = state.account_donate_balances.iter_mut().find(|(id, _)| id == &account_id) {
            balance.1 += amount;
        } else {
            state.account_donate_balances.push((account_id.clone(), amount));
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn distribute_tokens(ctx: Context<DistributeTokens>, distribution_tokens: DistributionTokens) -> Result<()> {
        let state = &mut ctx.accounts.state;

        require!(state.owner == *ctx.accounts.from.key, ErrorCode::MustBeOwner);

        let balance = state.account_donate_balances.iter().find(|(id, _)| id == &distribution_tokens.account_id).expect("Account not found").1;

        if balance == 0 { return Ok(());}

        for (account_id, p) in &distribution_tokens.distribution {
            if let Some(withdraw_balance) = state.account_withdraw_balances.iter_mut().find(|(id, _)| id == account_id) {
                let donate = (p * balance as f32) as u64;
                withdraw_balance.1 += donate;
            } else {
                let donate = (p * balance as f32) as u64;
                state.account_withdraw_balances.push((account_id.clone(), donate));
            }
        }

        if let Some(donate_balance) = state.account_donate_balances.iter_mut().find(|(id, _)| id == &distribution_tokens.account_id) {
            donate_balance.1 = 0;
        }

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, account_id: String) -> Result<()> {
        let state = &mut ctx.accounts.state;

        require!(state.owner == *ctx.accounts.from.key, ErrorCode::MustBeOwner);

        let mut balance = state.account_withdraw_balances.iter_mut().find(|(id, _)| id == &account_id).expect("Account not found").1;

        if balance == 0 { return Ok(());}

        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        balance = ((1.0 - COMMISSION) * balance as f64) as u64;
        token::transfer(cpi_ctx, balance)?;

        let withdraw_balance = state.account_withdraw_balances.iter_mut().find(|(id, _)| id == &account_id).expect("Account not found");
        withdraw_balance.1 = 0;

        Ok(())
    }
}
