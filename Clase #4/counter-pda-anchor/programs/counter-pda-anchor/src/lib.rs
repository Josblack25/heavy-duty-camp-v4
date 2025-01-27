use anchor_lang::prelude::*;
use crate::instructions::*;

mod collections;
mod instructions;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("3U9ToPFGTQ9j5uve2wij3hzE9QEd2iRaQgk5iRWE1sBE");

#[program]
mod counter_pda {
    use super::*;

    pub fn create_counter(ctx: Context<CreateCounter>) -> Result<()> {
        instructions::create::create_counter(ctx)
    }

    pub fn increment_counter(ctx: Context<IncrementCounter>) -> Result<()> {
        instructions::increment::increment_counter(ctx)
    }

}
