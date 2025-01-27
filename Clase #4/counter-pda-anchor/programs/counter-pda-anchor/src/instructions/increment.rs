use anchor_lang::prelude::*;
use crate::collections::*;

#[derive(Accounts)]
pub struct IncrementCounter<'info> {
    authority: Signer<'info>,

    #[account(
        mut,
        constraint = counter.authority == authority.key(),
        seeds = [
            Counter::COUNTER_SEED.as_ref(),
            authority.key().as_ref()
        ],
        bump = counter.bump
    )]
    counter: Account<'info, Counter>,
}

pub fn increment_counter(ctx: Context<IncrementCounter>) -> Result<()> {
    ctx.accounts.counter.count += 1;
    ctx.accounts.counter.increment();
    Ok(())
}
