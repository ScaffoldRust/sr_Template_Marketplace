# Escrow with Arbitration Smart Contract

A secure escrow system with built-in arbitration for marketplace transactions on Stellar blockchain using Rust and Soroban SDK.

## Overview

This smart contract provides a trustless escrow service where funds are held securely during buyer-seller transactions. In case of disputes, a designated arbitrator can resolve conflicts and determine the final outcome.

## Features

- **Secure Escrow Management**: Funds are held safely until transaction completion or dispute resolution
- **Built-in Arbitration**: Designated arbitrators can resolve disputes between buyers and sellers
- **Multi-token Support**: Works with any Stellar token via the Soroban token interface
- **Complete Audit Trail**: All actions are logged with detailed events
- **Role-based Access Control**: Strict authorization for all operations

## Quick Start

### Prerequisites

- Rust and Cargo
- Soroban CLI
- Stellar account with testnet funds

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd escrow-arbitration-contract

# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Deploy to Stellar testnet
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/escrow_arbitration.wasm --network testnet
```

## Usage

### 1. Create Escrow

```rust
let escrow_id = contract.create_escrow(
    buyer_address,
    seller_address,
    arbitrator_address,
    token_address,
    amount,
    "Purchase of digital goods".to_string()
);
```

### 2. Deposit Funds

```rust
contract.deposit(escrow_id, buyer_address);
```

### 3. Release Funds (Happy Path)

```rust
contract.release_funds(escrow_id, buyer_address);
```

### 4. Handle Disputes

```rust
// Raise dispute
contract.raise_dispute(escrow_id, disputer_address, "Item not as described".to_string());

// Arbitrator resolves dispute
contract.arbitrate(escrow_id, arbitrator_address, release_to_seller: false); // Refund buyer
```

## Contract States

| State | Description |
|-------|-------------|
| `Created` | Escrow created, awaiting deposit |
| `Funded` | Funds deposited, ready for release or dispute |
| `Disputed` | Dispute raised, awaiting arbitration |
| `Completed` | Funds released to seller or refunded to buyer |
| `Cancelled` | Escrow cancelled, funds refunded |

## API Reference

### Core Functions

- `create_escrow()` - Initialize new escrow transaction
- `deposit()` - Buyer deposits funds into escrow
- `release_funds()` - Release funds to seller (buyer only)
- `raise_dispute()` - Raise dispute for arbitration
- `arbitrate()` - Resolve dispute (arbitrator only)
- `refund()` - Process refund to buyer

### Query Functions

- `get_escrow()` - Retrieve escrow details
- `get_user_escrows()` - Get paginated list of user's escrows

## Security Features

- **Input Validation**: All parameters are validated before processing
- **Authorization Checks**: Role-based access control for all operations
- **State Validation**: Prevents invalid state transitions
- **No Direct Fund Access**: Funds can only be moved through defined workflows

## Events

The contract emits detailed events for monitoring and audit:

- `escrow_created` - New escrow created
- `deposited` - Funds deposited
- `funds_released` - Funds released to seller
- `dispute_raised` - Dispute initiated
- `arbitration_completed` - Dispute resolved
- `refunded` - Funds refunded to buyer

## Integration Example

```rust
use soroban_sdk::{Address, Env, String};

// Initialize contract
let contract = EscrowArbitrationContract::new(&env, contract_id);

// Create escrow
let escrow_id = contract.create_escrow(
    env.clone(),
    buyer,
    seller,
    arbitrator,
    token_address,
    1000u128,
    String::from_slice(&env, "Digital artwork purchase")
)?;

// Buyer deposits funds
contract.deposit(env.clone(), escrow_id, buyer)?;

// Seller delivers goods...

// Buyer releases funds
contract.release_funds(env.clone(), escrow_id, buyer)?;
```

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

Tests cover:
- Standard transaction flows
- Dispute scenarios
- Edge cases and error conditions
- Authorization checks
- Event emissions

## Architecture

```
├── lib.rs              # Contract interface and initialization
├── contract.rs         # Core business logic
├── escrow_storage.rs   # Data structures and storage
├── events.rs           # Event definitions and emission
├── error.rs            # Error types and handling
└── storage.rs          # Admin and configuration storage
```
