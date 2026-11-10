use anchor_lang::prelude::*;

use crate::{error::ErrorCode, state::Vault, VAULT_SEED};

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(init, payer = owner, space = 8 + Vault::INIT_SPACE, seeds = [VAULT_SEED, owner.key().as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
    ctx.accounts.vault.owner = ctx.accounts.owner.key();
    ctx.accounts.vault.bump = ctx.bumps.vault;
    Ok(())
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, seeds = [VAULT_SEED, owner.key().as_ref()], bump = vault.bump, has_one = owner)]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.owner.key(),
        &ctx.accounts.vault.key(),
        amount,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: receives lamports; must be the vault owner.
    #[account(mut, address = owner.key())]
    pub recipient: AccountInfo<'info>,
    #[account(mut, seeds = [VAULT_SEED, owner.key().as_ref()], bump = vault.bump, has_one = owner)]
    pub vault: Account<'info, Vault>,
}

pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    let rent_minimum = Rent::get()?.minimum_balance(8 + Vault::INIT_SPACE);
    let available = ctx
        .accounts
        .vault
        .to_account_info()
        .lamports()
        .saturating_sub(rent_minimum);
    require!(amount <= available, ErrorCode::InsufficientFunds);
    **ctx
        .accounts
        .vault
        .to_account_info()
        .try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.recipient.try_borrow_mut_lamports()? = ctx
        .accounts
        .recipient
        .lamports()
        .checked_add(amount)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    Ok(())
}

#[derive(Accounts)]
pub struct CloseVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: receives the closed vault's rent and remaining lamports.
    #[account(mut, address = owner.key())]
    pub recipient: AccountInfo<'info>,
    #[account(mut, close = recipient, seeds = [VAULT_SEED, owner.key().as_ref()], bump = vault.bump, has_one = owner)]
    pub vault: Account<'info, Vault>,
}

pub fn close_vault(_ctx: Context<CloseVault>) -> Result<()> {
    Ok(())
}
