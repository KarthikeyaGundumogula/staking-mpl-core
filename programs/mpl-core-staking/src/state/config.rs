use anchor_lang::prelude::*;
use mpl_core::types::OracleValidation;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub points_per_stake: u32,
    pub freeze_period: u8,
    pub burn_rewards:u32,
    pub rewards_bump: u8,
    pub config_bump: u8,
}

#[account]
pub struct Oracle {
    pub validation: OracleValidation,
    pub bump: u8,
    pub vault_bump: u8,
}
impl Space for Oracle {
    const INIT_SPACE: usize = 8 + 5 + 1;
}