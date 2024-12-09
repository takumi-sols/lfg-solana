use anchor_lang::prelude::*;

#[error_code]
pub enum BondingError {
    #[msg("The operation would exceed the bond cap.")]
    OverBondCap,

    #[msg("Bonding is currently closed.")]
    BondingClosed,
    
    #[msg("Invalid vesting period. Must be between 5 and 30 days.")]
    InvalidVestingPeriod,
    
    #[msg("Amount should be greater than the threshold")]
    AmountExceedsLimit,
    
    #[msg("Wrong USDC mint address")]
    InvalidMintAddress,
    
    #[msg("Calculation underflow error")]
    MathError,
    
    #[msg("No bond amount remaining")]
    NoBond,

    #[msg("Out of vesting period")]
    OutOfVestingPeriod
}
