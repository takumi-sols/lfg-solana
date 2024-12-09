use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::{AssociatedToken};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::mem::size_of;

declare_id!("BhURC7xGuDaucKUkpYhjRWktXyjYAhn6ZLCZj2KmZHAd");

const ACC_PRECISION: u128 = 100_000_000_000;

#[program]
pub mod lfg_staking {
    use super::*;

    pub fn create_state_first(
        _ctx: Context<CreateStateFirst>,
        token_per_second: u64,
    ) -> Result<()> {
        let state = &mut _ctx.accounts.state;
        state.authority = _ctx.accounts.authority.key();
        state.bump = _ctx.bumps.state;
        state.start_time = _ctx.accounts.clock.unix_timestamp;
        state.token_per_second = token_per_second;
        state.lfg_reward_vault = _ctx.accounts.lfg_reward_vault.key();        

        Ok(())
    }

    pub fn create_state_second(
        _ctx: Context<CreateStateSecond>
    ) -> Result<()> {
        let state = &mut _ctx.accounts.state;
        state.fee_vault = _ctx.accounts.fee_vault.key();

        Ok(())
    }
    
    pub fn fund_lfg_reward_token(_ctx: Context<Fund>, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: _ctx.accounts.lfg_user_vault.to_account_info(),
            to: _ctx.accounts.lfg_reward_vault.to_account_info(),
            authority: _ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = _ctx.accounts.lfg_token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn set_authority(ctx: Context<SetAuthority>, new_authority: Pubkey) -> Result<()> {
        let accts = ctx.accounts;
        let state = &mut accts.state;
    
        state.authority = new_authority;    
        Ok(())
    }

    pub fn change_tokens_per_second(
        _ctx: Context<ChangeTokensPerSecond>,
        token_per_second: u64,
    ) -> Result<()> {
        let state = &mut _ctx.accounts.state;

        let iter = &mut _ctx.remaining_accounts.iter();
        loop {
            if let Ok(pool_acc_info) = next_account_info(iter) {
                let mut pool = FarmPoolAccount::deserialize(&mut &pool_acc_info.try_borrow_data()?[8..])?;
                pool.update(&state, &_ctx.accounts.clock)?;
            } else {
                break;
            }
        }
        
        state.token_per_second = token_per_second;
        emit!(RateChanged {
            token_per_second
        });
        Ok(())
    }

    pub fn create_pool(
        _ctx: Context<CreateFarmPool>,
        point: u64,
        amount_multipler: u64,
    ) -> Result<()> {
        let state = &mut _ctx.accounts.state;
        
        let iter = &mut _ctx.remaining_accounts.iter();
        loop {
            if let Ok(pool_acc_info) = next_account_info(iter) {
                let mut pool = FarmPoolAccount::deserialize(&mut &pool_acc_info.try_borrow_data()?[8..])?;
                pool.update(&state, &_ctx.accounts.clock)?;
            } else {
                break;
            }
        }

        let pool = &mut _ctx.accounts.pool;
        pool.bump = _ctx.bumps.pool;
        pool.mint = _ctx.accounts.mint.key();
        pool.vault = _ctx.accounts.vault.key();
        pool.point = point;
        pool.lock_duration = 2 * 86400;
        pool.amount_multipler = amount_multipler;
        pool.authority = _ctx.accounts.authority.key();

        state.total_point = state.total_point.checked_add(point).unwrap();

        emit!(PoolCreated {
            pool: _ctx.accounts.pool.key(),
            mint: _ctx.accounts.mint.key()
        });
        Ok(())
    }

    pub fn change_pool_amount_multipler(
        _ctx: Context<ChangePoolSetting>,
        amount_multipler: u64,
    ) -> Result<()> {
        let pool = &mut _ctx.accounts.pool;
        pool.amount_multipler = amount_multipler;
        emit!(PoolAmountMultiplerChanged {
            pool: _ctx.accounts.pool.key(),
            amount_multipler
        });
        Ok(())
    }

    pub fn change_pool_point(_ctx: Context<ChangePoolSetting>, point: u64) -> Result<()> {
        let state = &mut _ctx.accounts.state;
        
        let iter = &mut _ctx.remaining_accounts.iter();
        loop {
            if let Ok(pool_acc_info) = next_account_info(iter) {
                let mut pool = FarmPoolAccount::deserialize(&mut &pool_acc_info.try_borrow_data()?[8..])?;
                pool.update(&state, &_ctx.accounts.clock)?;
            } else {
                break;
            }
        }

        let pool = &mut _ctx.accounts.pool;
        state.total_point = state
            .total_point
            .checked_sub(pool.point)
            .unwrap()
            .checked_add(point)
            .unwrap();
        pool.point = point;
        emit!(PoolPointChanged {
            pool: _ctx.accounts.pool.key(),
            point
        });
        Ok(())
    }

    pub fn create_user(_ctx: Context<CreatePoolUser>) -> Result<()> {
        let user = &mut _ctx.accounts.user;
        user.authority = _ctx.accounts.authority.key();
        user.bump = _ctx.bumps.user;
        user.pool = _ctx.accounts.pool.key();

        let pool = &mut _ctx.accounts.pool;
        pool.total_user += 1;
        emit!(UserCreated {
            pool: _ctx.accounts.pool.key(),
            user: _ctx.accounts.user.key(),
            authority: _ctx.accounts.authority.key(),
        });
        Ok(())
    }

    pub fn deposit(_ctx: Context<Stake>, amount: u64) -> Result<()> {
        let state = &mut _ctx.accounts.state;
        let user = &mut _ctx.accounts.user;
        let pool = &mut _ctx.accounts.pool;


        pool.update(&state, &_ctx.accounts.clock)?;
        user.calculate_lfg_reward_amount(&pool)?;

        user.amount = user.amount.checked_add(amount).unwrap();
        pool.amount = pool.amount.checked_add(amount).unwrap();

        user.calculate_lfg_reward_debt(&pool)?;
        user.last_stake_time = _ctx.accounts.clock.unix_timestamp;

        let cpi_accounts = Transfer {
            from: _ctx.accounts.user_vault.to_account_info(),
            to: _ctx.accounts.pool_vault.to_account_info(),
            authority: _ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = _ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        emit!(Deposit {
            pool: _ctx.accounts.pool.key(),
            user: _ctx.accounts.user.key(),
            authority: _ctx.accounts.authority.key(),
            amount
        });
        Ok(())
    }

    pub fn withdraw(_ctx: Context<Stake>, amount: u64) -> Result<()> {
        let state = &mut _ctx.accounts.state;
        let user = &mut _ctx.accounts.user;
        let pool = &mut _ctx.accounts.pool;

        require!(user.amount >= amount, ErrorCode::UnstakeOverAmount);
        let mut fee = 0;
        if user.last_stake_time
                .checked_add(pool.lock_duration)
                .unwrap()
                > _ctx.accounts.clock.unix_timestamp
        {
            fee = amount.checked_mul(10).unwrap().checked_div(100).unwrap();
        }
        let amount_out = amount - fee;

        pool.update(&state, &_ctx.accounts.clock)?;

        user.calculate_lfg_reward_amount(&pool)?;

        user.last_stake_time = _ctx.accounts.clock.unix_timestamp;
        user.amount = user.amount.checked_sub(amount).unwrap();
        pool.amount = pool.amount.checked_sub(amount).unwrap();

        user.calculate_lfg_reward_debt(&pool)?;
        drop(pool);

        let cpi_accounts = Transfer {
            from: _ctx.accounts.pool_vault.to_account_info(),
            to: _ctx.accounts.user_vault.to_account_info(),
            authority: _ctx.accounts.pool.to_account_info(),
        };

        let cpi_fee_accounts = Transfer {
            from: _ctx.accounts.pool_vault.to_account_info(),
            to: _ctx.accounts.fee_vault.to_account_info(),
            authority: _ctx.accounts.pool.to_account_info(),
        };

        let new_pool = &_ctx.accounts.pool;
        let seeds = &[new_pool.mint.as_ref(), &[new_pool.bump]];
        let signer = &[&seeds[..]];

        let cpi_program = _ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount_out)?;
        let cpi_fee_program = _ctx.accounts.token_program.to_account_info();
        let cpi_fee_ctx = CpiContext::new_with_signer(cpi_fee_program, cpi_fee_accounts, signer);
        token::transfer(cpi_fee_ctx, fee)?;
        emit!(Withdraw {
            pool: _ctx.accounts.pool.key(),
            user: _ctx.accounts.user.key(),
            authority: _ctx.accounts.authority.key(),
            amount
        });
        Ok(())
    }

    pub fn harvest(_ctx: Context<Harvest>) -> Result<()> {
        let state = &mut _ctx.accounts.state;
        let pool = &mut _ctx.accounts.pool;
        let user = &mut _ctx.accounts.user;

        pool.update(&state, &_ctx.accounts.clock)?;

        user.calculate_lfg_reward_amount(&pool)?;

        let total_lfg_reward = user.lfg_reward_amount.checked_add(user.lfg_extra_reward).unwrap().try_into().unwrap();

        let cpi_lfg_accounts = Transfer {
            from: _ctx.accounts.lfg_reward_vault.to_account_info(),
            to: _ctx.accounts.lfg_user_vault.to_account_info(),
            authority: state.to_account_info(),
        };

        let seeds = &[b"state".as_ref(), &[state.bump]];
        let signer = &[&seeds[..]];
        let cpi_lfg_program = _ctx.accounts.lfg_token_program.to_account_info();
        let cpi_lfg_ctx = CpiContext::new_with_signer(cpi_lfg_program, cpi_lfg_accounts, signer);

        token::transfer(cpi_lfg_ctx, total_lfg_reward)?;

        user.lfg_reward_amount = 0;
        user.lfg_extra_reward = 0;
        user.calculate_lfg_reward_debt(&pool)?;

        emit!(UserHarvested {
            pool: _ctx.accounts.pool.key(),
            user: _ctx.accounts.user.key(),
            authority: _ctx.accounts.authority.key(),
            lfg_amount: total_lfg_reward
        });
        Ok(())
    }

    pub fn recover_lfg_tokens(ctx: Context<RecoverLfgTokens>) -> Result<()> {
        let balance = ctx.accounts.lfg_vault.amount;
        
        let bump = ctx.bumps.state;    
        let state_seeds: &[&[&[u8]]] = &[&[&b"state".as_ref(), &[bump]]];
    
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.lfg_vault.to_account_info(),
            to: ctx.accounts.ata_to.to_account_info(),
            authority: ctx.accounts.state.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx.with_signer(state_seeds), balance as u64)?;

        Ok(())
    }

    pub fn recover_fee_tokens(ctx: Context<RecoverFeeTokens>) -> Result<()> {
        let balance = ctx.accounts.fee_vault.amount;
        
        let bump = ctx.bumps.state;    
        let fee_seeds: &[&[&[u8]]] = &[&[&b"state".as_ref(), &[bump]]];
    
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.fee_vault.to_account_info(),
            to: ctx.accounts.ata_to.to_account_info(),
            authority: ctx.accounts.state.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx.with_signer(fee_seeds), balance as u64)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateStateFirst<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        seeds = [b"state".as_ref()],
        bump,
        payer = authority,
        space = 8 + size_of::<StateAccount>()
    )]
    pub state: Account<'info, StateAccount>,

    pub lfg_reward_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        token::mint = lfg_reward_mint,
        token::authority = state,
        seeds = [b"vault".as_ref(), lfg_reward_mint.key().as_ref()],
        bump,
        payer = authority,
    )]
    pub lfg_reward_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct CreateStateSecond<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump,
        has_one = authority
    )]
    pub state: Account<'info, StateAccount>,

    pub lfg_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        token::mint = lfg_mint,
        token::authority = state,
        seeds = [b"fee".as_ref(), lfg_mint.key().as_ref()],
        bump,
        payer = authority,
    )]
    pub fee_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Fund<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump = state.bump
    )]
    pub state: Account<'info, StateAccount>,

    #[account(
        mut,
        constraint = lfg_reward_vault.owner == state.key()
    )]
    pub lfg_reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = lfg_user_vault.owner == authority.key()
    )]
    pub lfg_user_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = lfg_token_program.key == &token::ID
    )]
    pub lfg_token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SetAuthority<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, StateAccount>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct ChangeTokensPerSecond<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, StateAccount>,

    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct CreateFarmPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        seeds = [mint.key().as_ref()],
        bump,
        payer = authority,
        space = 8 + size_of::<FarmPoolAccount>()
    )]
    pub pool: Account<'info, FarmPoolAccount>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, StateAccount>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        associated_token::mint = mint,
        associated_token::authority = pool,
        payer = authority,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    #[account(
        constraint = token_program.key == &token::ID
    )]
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct CloseFarmPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, StateAccount>,

    #[account(
        mut,
        seeds = [pool.mint.key().as_ref()],
        bump = pool.bump,
        has_one = authority,
        close = authority
    )]
    pub pool: Account<'info, FarmPoolAccount>,

    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ChangePoolSetting<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump = state.bump
    )]
    pub state: Account<'info, StateAccount>,

    #[account(
        mut,
        seeds = [pool.mint.key().as_ref()],
        bump = pool.bump,
        has_one = authority
    )]
    pub pool: Account<'info, FarmPoolAccount>,

    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct CreatePoolUser<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        seeds = [pool.key().as_ref(), authority.key().as_ref()],
        bump,
        payer = authority,
        space = 8 + size_of::<FarmPoolUserAccount>()
    )]
    pub user: Account<'info, FarmPoolUserAccount>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump = state.bump
    )]
    pub state: Account<'info, StateAccount>,

    #[account(
        mut,
        seeds = [pool.mint.key().as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, FarmPoolAccount>,

    pub system_program: Program<'info, System>,

    #[account(
        constraint = token_program.key == &token::ID
    )]
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [pool.key().as_ref(), authority.key().as_ref()],
        bump = user.bump,
        has_one = pool,
        has_one = authority
    )]
    pub user: Account<'info, FarmPoolUserAccount>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump = state.bump
    )]
    pub state: Account<'info, StateAccount>,

    #[account(
        mut,
        seeds = [pool.mint.key().as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, FarmPoolAccount>,

    #[account(
        constraint = mint.key() == pool.mint
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut, 
        constraint = pool_vault.owner == pool.key()
    )]
    pub pool_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_vault.owner == authority.key()
    )]
    pub user_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = fee_vault.owner == state.key()
    )]
    pub fee_vault: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,

    #[account(
        constraint = token_program.key == &token::ID
    )]
    pub token_program: Program<'info, Token>,

    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct Harvest<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [pool.key().as_ref(), authority.key().as_ref()],
        bump = user.bump,
        has_one = pool,
        has_one = authority
    )]
    pub user: Account<'info, FarmPoolUserAccount>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump = state.bump
    )]
    pub state: Account<'info, StateAccount>,

    #[account(
        mut,
        seeds = [pool.mint.key().as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, FarmPoolAccount>,

    #[account(
        constraint = mint.key() == pool.mint
    )]
    pub mint: Box<Account<'info, Mint>>,
    
    #[account(
        mut,
        constraint = lfg_reward_vault.owner == state.key()
    )]
    pub lfg_reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = lfg_user_vault.owner == authority.key()
    )]
    pub lfg_user_vault: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,

    #[account(
        constraint = lfg_token_program.key == &token::ID
    )]
    pub lfg_token_program: Program<'info, Token>,

    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct RecoverLfgTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump
    )]
    pub state: Account<'info, StateAccount>,

    pub lfg_token_mint: Box<Account<'info, Mint>>,
    
    #[account(
        mut,
        token::mint = lfg_token_mint,
        token::authority = state,
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

#[derive(Accounts)]
pub struct RecoverFeeTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state".as_ref()],
        bump
    )]
    pub state: Account<'info, StateAccount>,

    pub lfg_token_mint: Box<Account<'info, Mint>>,
    
    #[account(
        mut,
        token::mint = lfg_token_mint,
        token::authority = state,
    )]
    pub fee_vault: Box<Account<'info, TokenAccount>>, 

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

#[account]
#[derive(Default)]
pub struct StateAccount {
    pub authority: Pubkey,
    pub lfg_reward_vault: Pubkey,
    pub fee_vault: Pubkey,
    pub bump: u8,
    pub total_point: u64,
    pub start_time: i64,
    pub token_per_second: u64,
}

#[account]
#[derive(Default)]
pub struct FarmPoolAccount {
    pub bump: u8,
    pub authority: Pubkey,
    pub amount: u64,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub point: u64,
    pub last_reward_time: i64,
    pub lock_duration: i64,
    pub acc_lfg_reward_per_share: u128,
    pub amount_multipler: u64,
    pub total_user: u64,
}

impl FarmPoolAccount {
    fn update<'info>(&mut self, state: &StateAccount, clock: &Sysvar<'info, Clock>) -> Result<()> {
        let seconds = u128::try_from(
            clock
                .unix_timestamp
                .checked_sub(self.last_reward_time)
                .unwrap(),
        )
        .unwrap();
        let mut lfg_reward_per_share: u128 = 0;
        if self.amount > 0 && seconds > 0 && self.point > 0 {
            lfg_reward_per_share = u128::from(state.token_per_second)
                .checked_mul(seconds)
                .unwrap()
                .checked_mul(u128::from(self.point))
                .unwrap()
                .checked_mul(ACC_PRECISION)
                .unwrap()
                .checked_div(u128::from(state.total_point))
                .unwrap()
                .checked_div(u128::from(self.amount))
                .unwrap();
        }
        self.acc_lfg_reward_per_share = self
            .acc_lfg_reward_per_share
            .checked_add(lfg_reward_per_share)
            .unwrap();
        self.last_reward_time = clock.unix_timestamp;

        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct FarmPoolUserAccount {
    pub bump: u8,
    pub pool: Pubkey,
    pub authority: Pubkey,
    pub amount: u64,
    pub lfg_reward_amount: u128,
    pub lfg_extra_reward: u128,
    pub lfg_reward_debt: u128,
    pub last_stake_time: i64,
}

impl FarmPoolUserAccount {
    fn calculate_lfg_reward_amount<'info>(
        &mut self,
        pool: &FarmPoolAccount,
    ) -> Result<()> {
        let lfg_pending_amount: u128 = u128::from(self.amount)
            .checked_mul(pool.acc_lfg_reward_per_share)
            .unwrap()
            .checked_div(ACC_PRECISION)
            .unwrap()
            .checked_sub(u128::from(self.lfg_reward_debt))
            .unwrap();
        self.lfg_reward_amount = self.lfg_reward_amount.checked_add(lfg_pending_amount).unwrap();
        Ok(())
    }
    fn calculate_lfg_reward_debt<'info>(&mut self, pool: &FarmPoolAccount) -> Result<()> {

        msg!("multiplied {}", u128::from(self.amount).checked_mul(pool.acc_lfg_reward_per_share).unwrap());
        msg!("scaled {}", u128::from(self.amount).checked_mul(pool.acc_lfg_reward_per_share).unwrap().checked_div(ACC_PRECISION).unwrap());

        self.lfg_reward_debt = u128::from(self.amount)
            .checked_mul(pool.acc_lfg_reward_per_share)
            .unwrap()
            .checked_div(ACC_PRECISION)
            .unwrap();
        Ok(())
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Over staked amount")]
    UnstakeOverAmount,
    #[msg("Pool is working")]
    WorkingPool,
    #[msg("Invalid Lock Duration")]
    InvalidLockDuration,
    #[msg("Invalid SEQ")]
    InvalidSEQ,
}
#[event]
pub struct RateChanged {
    token_per_second: u64,
}
#[event]
pub struct PoolCreated {
    pool: Pubkey,
    mint: Pubkey,
}
#[event]
pub struct PoolAmountMultiplerChanged {
    pool: Pubkey,
    amount_multipler: u64,
}
#[event]
pub struct PoolPointChanged {
    pool: Pubkey,
    point: u64,
}
#[event]
pub struct UserCreated {
    pool: Pubkey,
    user: Pubkey,
    authority: Pubkey,
}
#[event]
pub struct Deposit {
    pool: Pubkey,
    user: Pubkey,
    authority: Pubkey,
    amount: u64
}
#[event]
pub struct Withdraw {
    pool: Pubkey,
    user: Pubkey,
    authority: Pubkey,
    amount: u64,
}
#[event]
pub struct UserHarvested {
    pool: Pubkey,
    user: Pubkey,
    authority: Pubkey,
    lfg_amount: u64,
}
