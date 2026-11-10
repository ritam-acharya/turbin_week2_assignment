use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("The vault does not have enough withdrawable funds")]
    InsufficientFunds,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Expiry must be in the future")]
    InvalidExpiry,
    #[msg("Escrow has expired")]
    EscrowExpired,
    #[msg("Escrow has not expired")]
    EscrowNotExpired,
}
