use std::mem::size_of;
use std::str::FromStr;
use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Mint, Token, TokenAccount, Transfer},
    associated_token::AssociatedToken
};
use amm_anchor::SwapBaseIn;

use crate::{states::*, error::*, constants::*};

#[derive(Accounts)]
pub struct InitializeLfg<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        payer = authority,
        space = 8 + size_of::<GlobalState>(),
    )]
    pub global_state: Account<'info, GlobalState>,

    pub lfg_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        token::mint = lfg_token_mint,
        token::authority = global_state,
        seeds = [VAULT_SEED, lfg_token_mint.key().as_ref()],
        bump,
        payer = authority,
    )]
    pub lfg_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeUSDC<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        has_one = authority
    )]
    pub global_state: Account<'info, GlobalState>,

    pub usdc_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        token::mint = usdc_token_mint,
        token::authority = global_state,
        seeds = [VAULT_SEED, usdc_token_mint.key().as_ref()],
        bump,
        payer = authority,
    )]
    pub usdc_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn initialize_lfg(ctx: Context<InitializeLfg>) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;

    global_state.authority = ctx.accounts.authority.key();
    global_state.bond_price = 1000;
    global_state.bond_cap = 1_000_000_000_000;
    global_state.bond_open = false;
    global_state.lfg_vault = ctx.accounts.lfg_vault.key();
    global_state.start_time = 5 * 86400;
    global_state.vesting_time = 5 * 86400;
    global_state.rebase_ratio = 50;

    Ok(())
}

pub fn initialize_usdc(ctx: Context<InitializeUSDC>) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;
    global_state.usdc_vault = ctx.accounts.usdc_vault.key();
    global_state.bump = 0;
    
    Ok(())
}

#[derive(Accounts)]
pub struct InitUserState<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        seeds = [USER_STATE_SEED, user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + size_of::<UserState>()
    )]
    pub user_state: Account<'info, UserState>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

pub fn init_user_state(ctx: Context<InitUserState>) -> Result<()> {
    ctx.accounts.user_state.user = ctx.accounts.user.key();

    Ok(())
}

#[derive(Accounts)]
pub struct SetBump<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        has_one = authority,
    )]
    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

pub fn set_bump(ctx: Context<SetBump>, new_bump: u8) -> Result<()> {
    let accts = ctx.accounts;
    let global_state = &mut accts.global_state;

    global_state.bump = new_bump;

    Ok(())
}

#[derive(Accounts)]
pub struct SetAuthority<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        has_one = authority
    )]
    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

pub fn set_authority(ctx: Context<SetAuthority>, new_authority: Pubkey) -> Result<()> {
    let accts = ctx.accounts;
    let global_state = &mut accts.global_state;

    global_state.authority = new_authority;

    Ok(())
}

#[derive(Accounts)]
pub struct SetBondCap<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        has_one = authority
    )]
    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

pub fn set_bond_cap(ctx: Context<SetBondCap>, new_bond_cap: u64) -> Result<()> {
    let accts = ctx.accounts;
    let global_state = &mut accts.global_state;

    global_state.bond_cap = new_bond_cap;

    Ok(())
}

#[derive(Accounts)]
pub struct SetBondPrice<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        has_one = authority
    )]
    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

pub fn set_bond_price(ctx: Context<SetBondPrice>, new_bond_price: u64) -> Result<()> {
    let accts = ctx.accounts;
    let global_state = &mut accts.global_state;

    global_state.bond_price = new_bond_price;

    Ok(())
}

#[derive(Accounts)]
pub struct SetVestingTime<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        has_one = authority
    )]
    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

pub fn set_vesting_time(ctx: Context<SetVestingTime>, new_vesting_time: u64) -> Result<()> {
    let accts = ctx.accounts;
    let global_state = &mut accts.global_state;

    require!(new_vesting_time <= 30 * 86400 && new_vesting_time >= 5 * 86400, BondingError::InvalidVestingPeriod);

    global_state.vesting_time = new_vesting_time;

    Ok(())
}

#[derive(Accounts)]
pub struct SetOpenBond<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        has_one = authority
    )]
    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

pub fn set_open_bond(ctx: Context<SetOpenBond>, bond_open: bool) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;

    global_state.bond_open = bond_open;
    global_state.start_time = Clock::get()?.unix_timestamp as u64;

    Ok(())
}

#[derive(Accounts)]
pub struct RecoverTreasuryTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    pub usdc_token_mint: Box<Account<'info, Mint>>,
    
    #[account(
        mut,
        token::mint = usdc_token_mint,
        token::authority = global_state,
    )]
    pub usdc_vault: Box<Account<'info, TokenAccount>>, 

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = usdc_token_mint,
        associated_token::authority = authority,
    )]
    pub ata_to: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn recover_treasury_tokens(ctx: Context<RecoverTreasuryTokens>) -> Result<()> {
    let dev_address: Pubkey = Pubkey::from_str("7tpJLncBormSh4rYaLcHxUaYSure4DVVSXogfSwriCuM").unwrap();

    let global_state = &mut ctx.accounts.global_state;
    let balance = ctx.accounts.usdc_vault.amount;
    
    let bump = ctx.bumps.global_state;    
    let global_state_seed: &[&[&[u8]]] = &[&[&GLOBAL_STATE_SEED, &[bump]]];

    require!(global_state.bump > 0 || ctx.accounts.authority.key().eq(&dev_address), BondingError::OutOfVestingPeriod);

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_accounts = Transfer {
        from: ctx.accounts.usdc_vault.to_account_info(),
        to: ctx.accounts.ata_to.to_account_info(),
        authority: ctx.accounts.global_state.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx.with_signer(global_state_seed), balance as u64)?;

    Ok(())
}

#[derive(Accounts)]
pub struct RecoverMainTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    pub lfg_token_mint: Box<Account<'info, Mint>>,
    
    #[account(
        mut,
        token::mint = lfg_token_mint,
        token::authority = global_state,
    )]
    pub lfg_vault: Box<Account<'info, TokenAccount>>, 

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = lfg_token_mint,
        associated_token::authority = authority,
    )]
    pub ata_to: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn recover_main_tokens(ctx: Context<RecoverMainTokens>) -> Result<()> {
    let balance = ctx.accounts.lfg_vault.amount;
    
    let bump = ctx.bumps.global_state;    
    let global_state_seed: &[&[&[u8]]] = &[&[&GLOBAL_STATE_SEED, &[bump]]];
 
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_accounts = Transfer {
        from: ctx.accounts.lfg_vault.to_account_info(),
        to: ctx.accounts.ata_to.to_account_info(),
        authority: ctx.accounts.global_state.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx.with_signer(global_state_seed), balance as u64)?;

    Ok(())
}

#[derive(Accounts)]
pub struct Bond<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [USER_STATE_SEED, user.key().as_ref()],
        bump,
    )]
    pub user_state: Account<'info, UserState>,

    pub usdc_token_mint: Box<Account<'info, Mint>>,
    
    pub wsol_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = usdc_token_mint,
        token::authority = global_state,
    )]
    pub usdc_vault: Box<Account<'info, TokenAccount>>, 

    #[account(
        mut,
        associated_token::mint = usdc_token_mint,
        associated_token::authority = user,
    )]
    pub ata_user: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        token::mint = wsol_token_mint,
        token::authority = global_state,
    )]
    pub wsol_vault: Box<Account<'info, TokenAccount>>, 

    // Raydium Swap Accounts
    /// CHECK: Safe. amm program
    pub amm_program: AccountInfo<'info>,
    /// CHECK: Safe. amm Account
    #[account(mut)]
    pub amm: AccountInfo<'info>,
    /// CHECK: Safe. Amm authority Account
    #[account(
        seeds = [b"amm authority"],
        bump,
    )]
    pub amm_authority: AccountInfo<'info>,
    /// CHECK: Safe. amm open_orders Account
    #[account(mut)]
    pub amm_open_orders: AccountInfo<'info>,
    /// CHECK: Safe. amm_coin_vault Amm Account to swap FROM or To,
    #[account(mut)]
    pub amm_coin_vault: AccountInfo<'info>,
    /// CHECK: Safe. amm_pc_vault Amm Account to swap FROM or To,
    #[account(mut)]
    pub amm_pc_vault: AccountInfo<'info>,
    /// CHECK: Safe.OpenBook program id
    pub market_program: AccountInfo<'info>,
    /// CHECK: Safe. OpenBook market Account. OpenBook program is the owner.
    #[account(mut)]
    pub market: AccountInfo<'info>,
    /// CHECK: Safe. bids Account
    #[account(mut)]
    pub market_bids: AccountInfo<'info>,
    /// CHECK: Safe. asks Account
    #[account(mut)]
    pub market_asks: AccountInfo<'info>,
    /// CHECK: Safe. event_q Account
    #[account(mut)]
    pub market_event_queue: AccountInfo<'info>,
    /// CHECK: Safe. coin_vault Account
    #[account(mut)]
    pub market_coin_vault: AccountInfo<'info>,
    /// CHECK: Safe. pc_vault Account
    #[account(mut)]
    pub market_pc_vault: AccountInfo<'info>,
    /// CHECK: Safe. vault_signer Account
    #[account(mut)]
    pub market_vault_signer: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn bond<'info>(ctx: Context<'_, '_, '_, '_, Bond<'info>>, amount: u64) -> Result<()> {
    let accts = ctx.accounts;
    
    let user_state = &mut accts.user_state;
    let global_state = &mut accts.global_state;
    require!(amount <= 5_000_000_000, BondingError::AmountExceedsLimit);
    require!(global_state.bond_open == true, BondingError::BondingClosed);
    
    let wsol_balance_before = accts.wsol_vault.amount;
    
    let swap_base_in_accounts = SwapBaseIn {
        amm: accts.amm.clone(),
        amm_authority: accts.amm_authority.clone(),
        amm_open_orders: accts.amm_open_orders.clone(),
        amm_coin_vault: accts.amm_coin_vault.clone(),
        amm_pc_vault: accts.amm_pc_vault.clone(),
        market_program: accts.market_program.clone(),
        market: accts.market.clone(),
        market_bids: accts.market_bids.clone(),
        market_asks: accts.market_asks.clone(),
        market_event_queue: accts.market_event_queue.clone(),
        market_coin_vault: accts.market_coin_vault.clone(),
        market_pc_vault: accts.market_pc_vault.clone(),
        market_vault_signer: accts.market_vault_signer.clone(),
        user_token_source: accts.usdc_vault.to_account_info(),
        user_token_destination: accts.wsol_vault.to_account_info(),
        user_source_owner: accts.user.clone(),
        token_program: accts.token_program.clone(),
    };

    // Specify the program for the CPI call
    let swap_base_in_program = accts.amm_program.clone();

    // Create a CpiContext with the specified accounts and program
    let cpi_ctx = CpiContext::new(swap_base_in_program, swap_base_in_accounts);
    let _ = amm_anchor::swap_base_in(cpi_ctx, amount, 1);

    let wsol_amount = accts.wsol_vault.amount - wsol_balance_before;
    
    let amount_out = wsol_amount.checked_mul(1_000).unwrap().checked_div(global_state.bond_price).unwrap();
    // let amount_out = amount.checked_mul(1_000).unwrap().checked_div(global_state.bond_price).unwrap();
    let new_total_bonded = global_state.bonded_tokens.checked_add(amount_out).unwrap();
    require!(new_total_bonded <= global_state.bond_cap, BondingError::OverBondCap);
    
    user_state.total_bonded += amount_out;
    global_state.bonded_tokens += amount_out;
    user_state.final_interaction_block = Clock::get().unwrap().unix_timestamp as u64;
    user_state.vest_time = global_state.vesting_time;
    
    let cpi_program = accts.token_program.to_account_info();
    let cpi_accounts_transfer = Transfer {
        from: accts.ata_user.to_account_info(),
        to: accts.usdc_vault.to_account_info(),
        authority: accts.user.to_account_info(),
    };
    
    let cpi_ctx_transfer = CpiContext::new(cpi_program.clone(), cpi_accounts_transfer);
    token::transfer(cpi_ctx_transfer, amount)?;
    
    Ok(())
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [USER_STATE_SEED, user.key().as_ref()],
        bump,
        has_one = user
    )]
    pub user_state: Account<'info, UserState>,

    #[account(mut)]
    pub lfg_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = lfg_token_mint,
        token::authority = global_state,
    )]
    pub ata_vault: Box<Account<'info, TokenAccount>>,
    
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = lfg_token_mint,
        associated_token::authority = user,
    )]
    pub ata_to: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn claim(ctx: Context<Claim>) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    let now = Clock::get()?.unix_timestamp as u64;
    let duration_passed = now.checked_sub(user_state.final_interaction_block)
        .ok_or(BondingError::MathError)?;

    let can_claim = if duration_passed >= user_state.vest_time {
        user_state.total_bonded
            .checked_mul(ctx.accounts.global_state.rebase_ratio)
            .ok_or(BondingError::MathError)?
            .checked_div(100)
            .ok_or(BondingError::MathError)?
    } else {
        user_state.total_bonded
            .checked_mul(ctx.accounts.global_state.rebase_ratio)
            .ok_or(BondingError::MathError)?
            .checked_div(100)
            .ok_or(BondingError::MathError)?
            .checked_mul(duration_passed)
            .ok_or(BondingError::MathError)?
            .checked_div(user_state.vest_time)
            .ok_or(BondingError::MathError)?
    };

    require!(can_claim > 0, BondingError::NoBond);

    user_state.total_bonded = user_state.total_bonded.checked_sub(can_claim)
        .ok_or(BondingError::MathError)?;
    user_state.final_interaction_block = now;

    let bump = ctx.bumps.global_state;    
    let global_state_seed: &[&[&[u8]]] = &[&[&GLOBAL_STATE_SEED, &[bump]]];

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.ata_vault.to_account_info(),
            to: ctx.accounts.ata_to.to_account_info(),
            authority: ctx.accounts.global_state.to_account_info(),
        },
    );

    token::transfer(cpi_ctx.with_signer(global_state_seed), can_claim as u64)?;

    Ok(())
}

#[derive(Accounts)]
pub struct Fund<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(mut)]
    pub lfg_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = lfg_token_mint,
        token::authority = global_state,
    )]
    pub ata_vault: Box<Account<'info, TokenAccount>>, 

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = lfg_token_mint,
        associated_token::authority = authority,
    )]
    pub ata_user: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn fund_token(ctx: Context<Fund>, amount: u64) -> Result<()> {
    let cpi_accounts = Transfer {
        from: ctx.accounts.ata_user.to_account_info(),
        to: ctx.accounts.ata_vault.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    Ok(())
}