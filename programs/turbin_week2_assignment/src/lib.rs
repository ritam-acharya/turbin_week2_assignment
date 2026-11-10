pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
#[allow(ambiguous_glob_reexports)]
pub use instructions::*;
pub use state::{Escrow, Vault};

declare_id!("3yHH4ifAq949VERX5MxSjYSgR9iTfBCHsRnUFxbtzute");

#[program]
pub mod turbin_week2_assignment {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        instructions::vault::initialize_vault(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::vault::deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::vault::withdraw(ctx, amount)
    }

    pub fn close_vault(ctx: Context<CloseVault>) -> Result<()> {
        instructions::vault::close_vault(ctx)
    }

    pub fn make(
        ctx: Context<Make>,
        seed: u64,
        amount_a: u64,
        amount_b: u64,
        expiry: i64,
    ) -> Result<()> {
        instructions::escrow::make(ctx, seed, amount_a, amount_b, expiry)
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        instructions::escrow::take(ctx)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        instructions::escrow::refund(ctx)
    }

    pub fn update(ctx: Context<Update>, amount_b: u64, expiry: i64) -> Result<()> {
        instructions::escrow::update(ctx, amount_b, expiry)
    }
}
