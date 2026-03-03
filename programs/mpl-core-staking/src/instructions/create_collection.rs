use anchor_lang::prelude::*;
use mpl_core::{
    instructions::CreateCollectionV2CpiBuilder,
    types::{
        Attribute, Attributes, ExternalCheckResult, ExternalPluginAdapterInitInfo,
        HookableLifecycleEvent, OracleInitInfo, Plugin, PluginAuthority, PluginAuthorityPair,
        ValidationResultsOffset,
    },
    ID as MPL_CORE_ID,
};

use crate::state::Oracle as OracleAccount;

#[derive(Accounts)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"oracle",collection.key().as_ref()],
        bump = oracle.bump,
    )]
    pub oracle: Account<'info, OracleAccount>,
    /// CHECK: Collection account will be checked by the mpl core program
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: PDA update authority
    #[account(
    seeds= [b"update_authority",collection.key().as_ref()],
    bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: MPL CORE ID
    #[account(
      address = MPL_CORE_ID
    )]
    pub mpl_program: UncheckedAccount<'info>,
}

impl<'info> CreateCollection<'info> {
    pub fn create_collection(
        &mut self,
        name: String,
        uri: String,
        bumps: CreateCollectionBumps,
    ) -> Result<()> {
        let collection_addr = self.collection.key();

        let signer_seeds = &[
            b"update_authority",
            collection_addr.as_ref(),
            &[bumps.update_authority],
        ];

        CreateCollectionV2CpiBuilder::new(&self.mpl_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .payer(&self.payer.to_account_info())
            .update_authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .name(name)
            .uri(uri)
            .plugins(vec![PluginAuthorityPair {
                plugin: Plugin::Attributes(Attributes {
                    attribute_list: vec![
                        Attribute {
                            key: "rewards_deposited".to_string(),
                            value: "0".to_string(),
                        },
                        Attribute {
                            key: "total_staked".to_string(),
                            value: "0".to_string(),
                        },
                    ],
                }),
                authority: Some(PluginAuthority::UpdateAuthority),
            }])
            .external_plugin_adapters(vec![ExternalPluginAdapterInitInfo::Oracle(
                OracleInitInfo {
                    base_address: self.oracle.key(),
                    init_plugin_authority: None,
                    lifecycle_checks: vec![(
                        HookableLifecycleEvent::Transfer,
                        ExternalCheckResult { flags: 4 },
                    )],
                    base_address_config: None,
                    results_offset: Some(ValidationResultsOffset::Anchor),
                },
            )])
            .invoke_signed(&[signer_seeds])?;
        Ok(())
    }
}
