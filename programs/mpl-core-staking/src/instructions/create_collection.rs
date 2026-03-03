use anchor_lang::prelude::*;
use mpl_core::{instructions::CreateCollectionV2CpiBuilder, ID as MPL_CORE_ID};

#[derive(Accounts)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub collection: Signer<'info>,
    // CHECK: PDA update authority
    #[account(
    seeds= [b"update_authority",collection.key().as_ref()],
    bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    // CHECK: MPL CORE ID
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
            .invoke_signed(&[signer_seeds])?;
        Ok(())
    }
}
