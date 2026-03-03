use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to_checked, Mint, MintToChecked, TokenAccount, TokenInterface},
};
use mpl_core::{
    ID as MPL_CORE_ID, accounts::{BaseAssetV1, BaseCollectionV1}, fetch_plugin, instructions::{BurnV1CpiBuilder, UpdateCollectionPluginV1CpiBuilder, UpdatePluginV1CpiBuilder}, types::{Attribute, Attributes, FreezeDelegate, Plugin, PluginType, UpdateAuthority}
};

use crate::{constants::SECONDS_IN_A_DAY, errors::StakingError, state::Config};

#[derive(Accounts)]
pub struct BurnStakedNft<'info> {
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

impl<'info> BurnStakedNft<'info> {
    pub fn burn_staked_nft(&mut self, bumps: BurnStakedNftBumps) -> Result<()> {
        let current_timestamp = Clock::get()?.unix_timestamp;

        let base_asset = BaseAssetV1::try_from(&self.nft.to_account_info())?;

        let fetched_attribute_list = match fetch_plugin::<BaseAssetV1, Attributes>(
            &self.nft.to_account_info(),
            PluginType::Attributes,
        ) {
            Err(_) => return Err(StakingError::NotStaked.into()),
            Ok((_, attributes, _)) => attributes,
        };

        // Extract staked + staked_at values
        let mut staked_value: Option<String> = None;
        let mut staked_at_value: Option<String> = None;

        for attribute in &fetched_attribute_list.attribute_list {
            match attribute.key.as_str() {
                "staked" => staked_value = Some(attribute.value.clone()),
                "staked_at" => staked_at_value = Some(attribute.value.clone()),
                _ => {}
            }
        }

        require!(
            staked_value.as_deref() == Some("true"),
            StakingError::NotStaked
        );

        let staked_at_timestamp = staked_at_value
            .ok_or(StakingError::InvalidTimestamp)?
            .parse::<i64>()
            .map_err(|_| StakingError::InvalidTimestamp)?;

        let elapsed_seconds = current_timestamp
            .checked_sub(staked_at_timestamp)
            .ok_or(StakingError::InvalidTimestamp)?;

        let staked_days = elapsed_seconds
            .checked_div(SECONDS_IN_A_DAY)
            .ok_or(StakingError::InvalidTimestamp)?;

        // Burn bonus = days_staked * burn_rewards (set by admin in config)
        let amount = (staked_days as u64)
            .checked_mul(self.config.burn_rewards as u64)
            .ok_or(StakingError::Overflow)?;

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
        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
            .invoke_signed(&[&update_authority_seeds[..]])?;

        // Burn the NFT — update_authority signs to release the FreezeDelegate
        BurnV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.user.to_account_info()))
            .system_program(Some(&self.system_program.to_account_info()))
            .invoke_signed(&[&update_authority_seeds[..]])?;

        match fetch_plugin::<BaseCollectionV1, Attributes>(
            &self.collection.to_account_info(),
            PluginType::Attributes,
        ) {
            Ok((_, fetched_attributes_list, _)) => {
                let mut attribute_list: Vec<Attribute> =
                    Vec::with_capacity(fetched_attributes_list.attribute_list.len());
                for attribute in &fetched_attributes_list.attribute_list {
                    match attribute.key.as_str() {
                        "total_staked" => {
                            let total_staked = attribute.value.clone();
                            let total_staked: u32 = total_staked.parse().unwrap();
                            let new_total_staked = total_staked - 1;
                            attribute_list.push(Attribute {
                                key: "total_staked".to_string(),
                                value: new_total_staked.to_string(),
                            });
                        }
                        _ => {
                            attribute_list.push(attribute.clone());
                        }
                    }
                }

                UpdateCollectionPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                    .collection(&self.collection.to_account_info())
                    .payer(&self.user.to_account_info())
                    .plugin(Plugin::Attributes(Attributes { attribute_list }))
                    .authority(Some(&self.update_authority.to_account_info()))
                    .system_program(&self.system_program.to_account_info())
                    .invoke_signed(&[update_authority_seeds])?;
            }

            Err(_) => return Err(StakingError::NotStaked.into()),
        };

        // Mint bonus reward tokens to user's ATA
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
        
        require!(staked_days > 0, StakingError::FreezePeriodNotElapsed);
        require!(
            base_asset.owner == self.user.key(),
            StakingError::InvalidOwner
        );
        require!(
            base_asset.update_authority == UpdateAuthority::Collection(self.collection.key()),
            StakingError::InvalidAuthority
        );
        Ok(())
    }
}
