use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub points_per_stake: u32,
    pub freeze_period: u8,
    pub burn_rewards:u32,
    pub rewards_bump: u8,
    pub config_bump: u8,
}

// use mpl_core::types::{ExternalValidationResult, OracleValidation};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Eq, PartialEq)]
pub enum OracleValidation {
    Uninitialized,
    V1 {
        create: ExternalValidationResult,
        transfer: ExternalValidationResult,
        burn: ExternalValidationResult,
        update: ExternalValidationResult,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Eq, PartialEq)]
pub enum ExternalValidationResult {
    Approved,
    Rejected,
    Pass,
}

#[account]
pub struct Oracle {
    pub validation: OracleValidation,
    pub bump: u8,
    pub vault_bump: u8,
}

impl Space for Oracle {
    const INIT_SPACE: usize = 8 + 5 + 1 + 1;
}