use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub count: u64,
    pub authority: Pubkey,
    pub bump: u8,
}

impl Counter {
    pub const COUNTER_SEED: &'static str = "counter";

    pub fn increment(&mut self) {
        self.count += 1;
    }
}
