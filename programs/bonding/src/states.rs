use anchor_lang::prelude::*;

#[account]
pub struct GlobalState {
    pub authority: Pubkey,
    pub bond_price: u64,
    pub bond_cap: u64,
    pub bonded_tokens: u64,
    pub lfg_vault: Pubkey,
    pub usdc_vault: Pubkey,
    pub start_time: u64,
    pub vesting_time: u64,
    pub bond_open: bool,
    pub rebase_ratio: u64,
    pub bump: u8,
}

#[account]
pub struct UserState {
    pub user: Pubkey,
    pub total_bonded: u64,
    pub final_interaction_block: u64,
    pub vest_time: u64,
}
