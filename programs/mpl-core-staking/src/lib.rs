use anchor_lang::prelude::*;

mod instructions;
mod errors;
mod state;
use instructions::*;

declare_id!("CBA2DLyUHWP4ApZnVs9xCSWTQs95xyDbwyyVbGmz2e68");

#[program]
pub mod mpl_core_staking {
    use super::*;

    pub fn create_collection(ctx:Context<CreateCollection>,name:String,uri:String) -> Result<()> {
        ctx.accounts.create_collection(name, uri, ctx.bumps)
    }

    pub fn init_config(ctx: Context<InitConfig>, points_per_stake: u32, freeze_period: u8) -> Result<()> {
        ctx.accounts.init_config(points_per_stake, freeze_period, &ctx.bumps)
    }

     pub fn mint_nft(ctx: Context<Mint>, name: String, uri: String) -> Result<()> {
        ctx.accounts.mint_nft(name, uri, &ctx.bumps)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.stake(&ctx.bumps)
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        ctx.accounts.unstake(&ctx.bumps)
    }
}
