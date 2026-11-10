use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount, TransferChecked};

use crate::{error::ErrorCode, state::Escrow, ESCROW_SEED};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,
    #[account(mut, token::mint = mint_a, token::authority = maker)]
    pub maker_token_a: Account<'info, TokenAccount>,
    #[account(init, payer = maker, space = 8 + Escrow::INIT_SPACE, seeds = [ESCROW_SEED, maker.key().as_ref(), &seed.to_le_bytes()], bump)]
    pub escrow: Account<'info, Escrow>,
    #[account(init, payer = maker, token::mint = mint_a, token::authority = escrow)]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn make(
    ctx: Context<Make>,
    seed: u64,
    amount_a: u64,
    amount_b: u64,
    expiry: i64,
) -> Result<()> {
    require!(amount_a > 0 && amount_b > 0, ErrorCode::InvalidAmount);
    require!(
        expiry > Clock::get()?.unix_timestamp,
        ErrorCode::InvalidExpiry
    );
    ctx.accounts.escrow.set_inner(Escrow {
        seed,
        maker: ctx.accounts.maker.key(),
        mint_a: ctx.accounts.mint_a.key(),
        mint_b: ctx.accounts.mint_b.key(),
        amount_a,
        amount_b,
        expiry,
        bump: ctx.bumps.escrow,
    });
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.maker_token_a.to_account_info(),
                mint: ctx.accounts.mint_a.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.maker.to_account_info(),
            },
        ),
        amount_a,
        ctx.accounts.mint_a.decimals,
    )
}

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    /// CHECK: validated against escrow maker and receives the agreed tokens.
    #[account(mut, address = escrow.maker)]
    pub maker: AccountInfo<'info>,
    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,
    #[account(mut, token::mint = mint_b, token::authority = taker)]
    pub taker_token_b: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_b, token::authority = maker)]
    pub maker_token_b: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_a, token::authority = taker)]
    pub taker_token_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, close = maker, has_one = maker, has_one = mint_a, has_one = mint_b, seeds = [ESCROW_SEED, maker.key().as_ref(), &escrow.seed.to_le_bytes()], bump = escrow.bump)]
    pub escrow: Box<Account<'info, Escrow>>,
    #[account(mut, token::mint = mint_a, token::authority = escrow)]
    pub vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

pub fn take(ctx: Context<Take>) -> Result<()> {
    require!(
        Clock::get()?.unix_timestamp < ctx.accounts.escrow.expiry,
        ErrorCode::EscrowExpired
    );
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.taker_token_b.to_account_info(),
                mint: ctx.accounts.mint_b.to_account_info(),
                to: ctx.accounts.maker_token_b.to_account_info(),
                authority: ctx.accounts.taker.to_account_info(),
            },
        ),
        ctx.accounts.escrow.amount_b,
        ctx.accounts.mint_b.decimals,
    )?;
    let maker_key = ctx.accounts.escrow.maker;
    let seed_bytes = ctx.accounts.escrow.seed.to_le_bytes();
    let bump = [ctx.accounts.escrow.bump];
    let signer: &[&[u8]] = &[ESCROW_SEED, maker_key.as_ref(), &seed_bytes, &bump];
    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.mint_a.to_account_info(),
                to: ctx.accounts.taker_token_a.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            &[signer],
        ),
        ctx.accounts.vault.amount,
        ctx.accounts.mint_a.decimals,
    )?;
    token::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.maker.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        },
        &[signer],
    ))
}

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: Account<'info, Mint>,
    #[account(mut, token::mint = mint_a, token::authority = maker)]
    pub maker_token_a: Account<'info, TokenAccount>,
    #[account(mut, close = maker, has_one = maker, has_one = mint_a, seeds = [ESCROW_SEED, maker.key().as_ref(), &escrow.seed.to_le_bytes()], bump = escrow.bump)]
    pub escrow: Account<'info, Escrow>,
    #[account(mut, token::mint = mint_a, token::authority = escrow)]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn refund(ctx: Context<Refund>) -> Result<()> {
    require!(
        Clock::get()?.unix_timestamp >= ctx.accounts.escrow.expiry,
        ErrorCode::EscrowNotExpired
    );
    let maker_key = ctx.accounts.escrow.maker;
    let seed_bytes = ctx.accounts.escrow.seed.to_le_bytes();
    let bump = [ctx.accounts.escrow.bump];
    let signer: &[&[u8]] = &[ESCROW_SEED, maker_key.as_ref(), &seed_bytes, &bump];
    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.mint_a.to_account_info(),
                to: ctx.accounts.maker_token_a.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            &[signer],
        ),
        ctx.accounts.vault.amount,
        ctx.accounts.mint_a.decimals,
    )?;
    token::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.maker.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        },
        &[signer],
    ))
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(mut, has_one = maker, seeds = [ESCROW_SEED, maker.key().as_ref(), &escrow.seed.to_le_bytes()], bump = escrow.bump)]
    pub escrow: Account<'info, Escrow>,
}

pub fn update(ctx: Context<Update>, amount_b: u64, expiry: i64) -> Result<()> {
    require!(amount_b > 0, ErrorCode::InvalidAmount);
    require!(
        Clock::get()?.unix_timestamp < ctx.accounts.escrow.expiry,
        ErrorCode::EscrowExpired
    );
    require!(
        expiry > Clock::get()?.unix_timestamp,
        ErrorCode::InvalidExpiry
    );
    ctx.accounts.escrow.amount_b = amount_b;
    ctx.accounts.escrow.expiry = expiry;
    Ok(())
}
