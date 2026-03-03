use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to_checked, Mint, MintToChecked, TokenAccount, TokenInterface},
};
use mpl_core::{
    accounts::BaseAssetV1,
    fetch_plugin,
    instructions::UpdatePluginV1CpiBuilder,
    types::{Attribute, Attributes, Plugin, PluginType},
    ID as MPL_CORE_ID,
};

use crate::{errors::StakingError, state::Config};

const SECONDS_PER_DAY: i64 = 86400;

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: PDA Update authority
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [b"rewards", config.key().as_ref()],
        bump = config.rewards_bump
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = rewards_mint,
        associated_token::authority = user,
    )]
    pub user_rewards_ata: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: NFT account will be checked by the mpl core program
    #[account(mut)]
    pub nft: UncheckedAccount<'info>,
    /// CHECK: Collection account will be checked by the mpl core program
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: This is the ID of the Metaplex Core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ClaimRewards<'info> {
    pub fn claim(&mut self, bumps: ClaimRewardsBumps) -> Result<()> {
        let current_timestamp = Clock::get()?.unix_timestamp;

        // Fetch the NFT's attribute plugin — error if not staked
        let fetched_attribute_list = match fetch_plugin::<BaseAssetV1, Attributes>(
            &self.nft.to_account_info(),
            PluginType::Attributes,
        ) {
            Err(_) => return Err(StakingError::NotStaked.into()),
            Ok((_, attributes, _)) => attributes,
        };

        let mut attribute_list: Vec<Attribute> =
            Vec::with_capacity(fetched_attribute_list.attribute_list.len());
        let mut staked_value: Option<String> = None;
        let mut last_claimed_at_value: Option<String> = None;

        for attribute in &fetched_attribute_list.attribute_list {
            match attribute.key.as_str() {
                "staked" => {
                    staked_value = Some(attribute.value.clone());
                    attribute_list.push(attribute.clone());
                }
                "staked_at" => {
                    attribute_list.push(attribute.clone()); // never touched
                }
                "last_claimed_at" => {
                    last_claimed_at_value = Some(attribute.value.clone());
                    attribute_list.push(Attribute {
                        key: "last_claimed_at".to_string(),
                        value: current_timestamp.to_string(),
                    });
                }
                _ => {
                    attribute_list.push(attribute.clone());
                }
            }
        }

        require!(
            staked_value.as_deref() == Some("true"),
            StakingError::NotStaked
        );

        let claim_from_timestamp = last_claimed_at_value
            .ok_or(StakingError::InvalidTimestamp)?
            .parse::<i64>()
            .map_err(|_| StakingError::InvalidTimestamp)?;

        let elapsed_seconds = current_timestamp
            .checked_sub(claim_from_timestamp)
            .ok_or(StakingError::InvalidTimestamp)?;

        let elapsed_days = elapsed_seconds
            .checked_div(SECONDS_PER_DAY)
            .ok_or(StakingError::InvalidTimestamp)?;

        require!(elapsed_days > 0, StakingError::FreezePeriodNotElapsed);

        let amount = (elapsed_days as u64)
            .checked_mul(self.config.points_per_stake as u64)
            .ok_or(StakingError::Overflow)?;

        // Signer seeds
        let collection_key = self.collection.key();
        let update_authority_seeds = &[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ];
        let config_seeds = &[
            b"config",
            collection_key.as_ref(),
            &[self.config.config_bump],
        ];
        let config_seeds = [&config_seeds[..]];

        // Update NFT attributes (reset last_claimed_at to now)
        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::Attributes(Attributes { attribute_list }))
            .invoke_signed(&[&update_authority_seeds[..]])?;

        // Mint reward tokens to user's ATA
        let cpi_accounts = MintToChecked {
            mint: self.rewards_mint.to_account_info(),
            to: self.user_rewards_ata.to_account_info(),
            authority: self.config.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            &config_seeds,
        );
        mint_to_checked(cpi_ctx, amount, self.rewards_mint.decimals)?;

        Ok(())
    }
}