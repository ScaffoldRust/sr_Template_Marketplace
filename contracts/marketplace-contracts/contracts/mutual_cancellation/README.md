# Mutual Cancellation Smart Contract

A Soroban-based smart contract that enables buyers and sellers to mutually agree on cancelling marketplace transactions and returning escrowed funds.

## Overview

The Mutual Cancellation contract provides a secure and transparent way to handle transaction cancellations in a marketplace. It ensures that:

1. Either party (buyer or seller) can propose a cancellation
2. Funds are only returned when both parties explicitly agree to the cancellation
3. Proposals expire after a configurable time window if the counterparty doesn't respond
4. All actions are recorded on-chain for transparency and auditability

## Contract Functionality

### Initialize

Sets up the contract with a configurable response window for cancellation proposals.

### Create Transaction

Creates a new transaction in escrow. The buyer transfers funds to the contract, which are held until the transaction is completed or cancelled.

### Propose Cancellation

Either the buyer or seller can propose cancellation of a transaction. This initiates a cancellation request that the counterparty can agree to.

### Agree to Cancellation

The counterparty (not the proposer) can agree to a cancellation proposal, which will return the escrowed funds to the buyer and mark the transaction as cancelled.

### Check Cancellation Expiry

Checks if a cancellation proposal has expired (passed the response window).

### Reset Expired Proposal

Resets an expired cancellation proposal, allowing either party to make a new proposal.

## Events

The contract emits the following events for transparency:

1. **Transaction Created** - When a new transaction is created
2. **Cancellation Proposed** - When either party proposes a cancellation
3. **Cancellation Agreed** - When both parties agree and funds are returned
4. **Cancellation Expired** - When a cancellation proposal expires

## Usage Examples

### Creating a Transaction

```rust
// As a buyer
client.create_transaction(&buyer_address, &seller_address, &token_address, &amount);
```

### Proposing Cancellation

```rust
// Buyer proposes
client.buyer_propose_cancellation(&transaction_id);

// Seller proposes
client.seller_propose_cancellation(&transaction_id);
```

### Agreeing to Cancellation

```rust
// Counterparty (not the proposer) calls this
client.agree_to_cancellation(&transaction_id);
```

### Checking Expiration

```rust
let is_expired = client.check_cancellation_expiry(&transaction_id);
if is_expired {
    client.reset_expired_proposal(&transaction_id);
}
```

## Security Considerations

- The contract validates that only authorized parties (buyer or seller) can propose or agree to cancellations
- Funds are securely held in escrow until both parties agree to cancel
- Cancellation proposals expire if not acted upon, preventing transactions from being locked indefinitely
- All operations require proper authentication via `require_auth()`

## Building and Testing

```bash
cd contracts/marketplace-contracts
cargo build --target wasm32-unknown-unknown --release
cargo test
``` 