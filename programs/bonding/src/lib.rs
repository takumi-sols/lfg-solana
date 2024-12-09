use anchor_lang::prelude::*;

pub mod constants;
pub mod states;
pub mod instructions;
pub mod error;

use instructions::*;

declare_id!("ZbZYTQmYDJ8K6NavJaYz1NpFqF6tk5aDfrup5s9dkpJ");

#[program]
pub mod solana_lfg {
    use super::*;

    pub fn initialize_lfg(ctx: Context<InitializeLfg>) -> Result<()> {
        instructions::initialize_lfg(ctx)
    }

    pub fn initialize_usdc(ctx: Context<InitializeUSDC>) -> Result<()> {
        instructions::initialize_usdc(ctx)
    }

    pub fn init_user_state(ctx: Context<InitUserState>) -> Result<()> {
        instructions::init_user_state(ctx)
    }

    pub fn set_bump(ctx: Context<SetBump>, new_bump: u8) -> Result<()> {
        instructions::set_bump(ctx, new_bump)
    }

    pub fn set_authority(ctx: Context<SetAuthority>, new_authority: Pubkey) -> Result<()> {
        instructions::set_authority(ctx, new_authority)
    }

    pub fn set_bond_cap(ctx: Context<SetBondCap>, new_bond_cap: u64) -> Result<()> {
        instructions::set_bond_cap(ctx, new_bond_cap)
    }

    pub fn set_bond_price(ctx: Context<SetBondPrice>, new_bond_price: u64) -> Result<()> {
        instructions::set_bond_price(ctx, new_bond_price)
    }

    pub fn set_vesting_time(ctx: Context<SetVestingTime>, new_vesting_time: u64) -> Result<()> {
        instructions::set_vesting_time(ctx, new_vesting_time)
    }

    pub fn set_open_bond(ctx: Context<SetOpenBond>, bond_open: bool) -> Result<()> {
        instructions::set_open_bond(ctx, bond_open)
    }

    pub fn recover_treasury_tokens(ctx: Context<RecoverTreasuryTokens>) -> Result<()> {
        instructions::recover_treasury_tokens(ctx)
    }

    pub fn recover_main_tokens(ctx: Context<RecoverMainTokens>) -> Result<()> {
        instructions::recover_main_tokens(ctx)
    }

    pub fn bond(ctx: Context<Bond>, amount: u64) -> Result<()> {
        instructions::bond(ctx, amount)
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        instructions::claim(ctx)
    }

    pub fn fund_token(ctx: Context<Fund>, amount: u64) -> Result<()> {
        instructions::fund_token(ctx, amount)
    }
}
