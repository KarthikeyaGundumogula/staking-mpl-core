use anchor_lang::prelude::*;

mod errors;
mod instructions;
mod state;
use instructions::*;

declare_id!("DsQFzrNNWqcpdZqNcQJjt7anpTBeBfmR3tW8JwpLWzAD");

#[program]
pub mod mpl_core_staking {
    use super::*;

    pub fn create_collection(
        ctx: Context<CreateCollection>,
        name: String,
        uri: String,
    ) -> Result<()> {
        ctx.accounts.create_collection(name, uri, ctx.bumps)
    }

    pub fn init_config(
        ctx: Context<InitConfig>,
        points_per_stake: u32,
        freeze_period: u8,
        burn_rewards: u32,
    ) -> Result<()> {
        ctx.accounts
            .init_config(points_per_stake, freeze_period, burn_rewards, &ctx.bumps)
    }

    pub fn mint_nft(ctx: Context<Mint>, name: String, uri: String) -> Result<()> {
        ctx.accounts.mint_nft(name, uri, &ctx.bumps)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.stake(&ctx.bumps)
    }

    pub fn burn_staked_nft(ctx:Context<BurnStakedNft>) -> Result<()> {
        ctx.accounts.burn_staked_nft(ctx.bumps)
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        ctx.accounts.claim(ctx.bumps)
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        ctx.accounts.unstake(&ctx.bumps)
    }
}
