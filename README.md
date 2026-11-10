# Turbin Week 2 Assignment

An [Anchor](https://www.anchor-lang.com/) program on Solana that implements:

1. An owner-controlled SOL vault.
2. A timed SPL-token escrow.

The program is configured for Solana localnet and uses Anchor `0.32.1`.

## Program ID

```text
3yHH4ifAq949VERX5MxSjYSgR9iTfBCHsRnUFxbtzute
```

## Features

### SOL vault

Each owner can create one vault PDA. The vault account stores its owner and
holds the owner's deposited lamports.

| Instruction | Description |
| --- | --- |
| `initializeVault` | Creates the owner's vault PDA. |
| `deposit(amount)` | Transfers lamports from the owner into the vault. |
| `withdraw(amount)` | Returns lamports to the owner while preserving the vault's rent-exempt balance. |
| `closeVault` | Closes the vault and sends its remaining lamports to the owner. |

The vault PDA is derived from:

```text
["vault", owner_public_key]
```

Only the vault owner can deposit, withdraw, or close the vault. Deposit and
withdraw amounts must be greater than zero.

### Timed SPL-token escrow

An escrow maker deposits a fixed amount of mint A into a token account
controlled by an escrow PDA. Before the expiry timestamp, a taker can exchange
the deposited mint A for the agreed amount of mint B. If the expiry is reached
first, the maker can refund the deposited mint A.

| Instruction | Description |
| --- | --- |
| `make(seed, amountA, amountB, expiry)` | Creates an escrow and deposits mint A. |
| `update(amountB, expiry)` | Allows the maker to change the requested mint B amount and expiry while the escrow is open. |
| `take` | Transfers mint B from the taker to the maker and releases mint A to the taker. |
| `refund` | Returns mint A to the maker after expiry. |

The escrow PDA is derived from:

```text
["escrow", maker_public_key, seed_as_little_endian_u64]
```

Both mints and all token accounts are validated by the program. The escrow uses
the classic SPL Token program (`anchor-spl`'s `token` feature), not Token-2022.

## Repository layout

```text
.
├── Anchor.toml
├── programs/
│   └── turbin_week2_assignment/
│       └── src/
│           ├── instructions/
│           │   ├── escrow.rs
│           │   └── vault.rs
│           ├── state/
│           │   ├── escrow.rs
│           │   └── vault.rs
│           ├── constants.rs
│           ├── error.rs
│           └── lib.rs
└── tests/
    └── turbin_week2_assignment.ts
```

## Prerequisites

Install the following tools before working with the project:

- Rust and Cargo
- Solana CLI
- Anchor CLI `0.32.1`
- Node.js and Yarn

Verify the important versions:

```bash
anchor --version
solana --version
node --version
yarn --version
```

The Anchor provider uses the local Solana cluster and the default wallet at
`~/.config/solana/id.json`. Ensure that wallet exists and is funded for local
development.

## Installation

Install JavaScript dependencies from the repository root:

```bash
yarn install
```

Build the Solana program:

```bash
anchor build
```

## Running the tests

Run the complete localnet workflow:

```bash
anchor test
```

This command builds the program, starts a local validator, deploys the program,
and runs the TypeScript tests configured in `Anchor.toml`.

The test suite verifies:

- Vault initialization, deposit, withdrawal, and closure.
- Rejection of zero-value deposits.
- Escrow creation and token funding.
- Escrow offer updates.
- Successful token exchange before expiry.
- Refund after expiry.

The refund test intentionally waits for the short test expiry to pass.

To run only the TypeScript test command against an already-running validator
and deployed program:

```bash
yarn ts-mocha -p ./tsconfig.json -t 1000000 "tests/**/*.ts"
```

Do not use `--skip-deploy` unless the current local validator already contains
the program deployment.

## Linting

Check formatting:

```bash
yarn lint
```

Apply the repository's Prettier formatting:

```bash
yarn lint:fix
```

## Error conditions

The program reports these custom errors:

- `InvalidAmount`: an amount must be greater than zero.
- `InsufficientFunds`: a vault does not have enough lamports available after
  preserving rent exemption.
- `ArithmeticOverflow`: a lamport calculation overflowed.
- `InvalidExpiry`: an escrow expiry must be in the future when created or
  updated.
- `EscrowExpired`: a taker attempted to accept an expired escrow.
- `EscrowNotExpired`: a maker attempted to refund an escrow before expiry.

## Development notes

- The SOL vault is a custodial lamport balance PDA; it is not an in-kind market
  position vault.
- The escrow's mint A amount is fixed after `make`.
- `take` and `refund` close the escrow token account and the escrow state
  account after completing their respective transfers.
- Amounts are expressed in the smallest unit of the relevant asset. For
  example, a six-decimal SPL mint amount of `1_000_000` represents one token.
- Generated build artifacts are produced under `target/` and the local test
  validator ledger under `.anchor/test-ledger/`.
