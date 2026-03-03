use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};
use mpl_core::accounts::BaseCollectionV1;

use crate::{errors::StakingError, state::Config};

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: Validated by the mpl core
    pub collection: UncheckedAccount<'info>,
    /// CHECK: PDA derivation validated by runtime 
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    #[account(
        init, 
        payer = admin, 
        space = 8 + Config::INIT_SPACE, 
        seeds = [b"config", collection.key().as_ref()], 
        bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = config,
        seeds = [b"rewards", config.key().as_ref()],
        bump
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl InitConfig<'_> {
    pub fn init_config(&mut self, points_per_stake: u32, freeze_period: u8,burn_rewards:u32, bumps: &InitConfigBumps) -> Result<()> {
        // Validate collection account
        let base_collection = BaseCollectionV1::try_from(&self.collection.to_account_info())?;
        require!(base_collection.update_authority == self.update_authority.key(), StakingError::InvalidAuthority);

        self.config.set_inner(Config {
            points_per_stake, 
            freeze_period, 
            burn_rewards,
            rewards_bump: bumps.rewards_mint, 
            config_bump: bumps.config });
        Ok(())
    }
}
