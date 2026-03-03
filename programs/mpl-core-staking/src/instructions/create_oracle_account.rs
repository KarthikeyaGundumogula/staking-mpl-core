use anchor_lang::prelude::*;
use mpl_core::types::{ExternalValidationResult, OracleValidation};

use crate::{state::Oracle,helpers::is_market_open};

#[derive(Accounts)]
pub struct CreateOracle<'info> {
    pub signer: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = Oracle::INIT_SPACE,
        seeds = [b"oracle",collection.key().as_ref()],
        bump
    )]
    pub oracle: Account<'info, Oracle>,
    /// CHECK: Collection account will be checked by the mpl core program
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    #[account(
        seeds = [b"reward_vault", oracle.key().as_ref()],
        bump,
    )]
    pub reward_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateOracle<'info>{
     pub fn create(&mut self,bumps:CreateOracleBumps) -> Result<()> {
        // Set the Oracle validation based on the time and if the US market is open
        match is_market_open(Clock::get()?.unix_timestamp) {
            true => {
                self.oracle.set_inner(
                    Oracle {
                        validation: OracleValidation::V1 {
                            transfer: ExternalValidationResult::Approved,
                            create: ExternalValidationResult::Pass,
                            update: ExternalValidationResult::Pass,
                            burn: ExternalValidationResult::Pass,
                        },
                        bump: bumps.oracle,
                        vault_bump: bumps.reward_vault,
                    }
                );
            }
            false => {
                self.oracle.set_inner(
                    Oracle {
                        validation: OracleValidation::V1 {
                            transfer: ExternalValidationResult::Rejected,
                            create: ExternalValidationResult::Pass,
                            update: ExternalValidationResult::Pass,
                            burn: ExternalValidationResult::Pass,
                        },
                        bump: bumps.oracle,
                        vault_bump: bumps.reward_vault,
                    }
                );
            }
        }

        Ok(())
    }
}
