# Staggered Payment Smart Contract for a Marketplace

A Soroban-based smart contract that enables buyers and sellers which facilitates incremental payments to sellers based on predefined milestones.

## Overview

This project implements a Staggered Payment smart contract using Rust and the Soroban SDK. Designed for a Stellar-based marketplace, this contract facilitates incremental payments to sellers based on predefined milestones. This ensures that sellers are compensated progressively as work is completed or conditions are met, providing a secure and transparent payment mechanism for complex projects or services.

## Objectives

- **Fund Commitment**: Allow buyers to commit the full payment amount upfront, securing it in an escrow.

- **Incremental Payments**: Enable sellers to receive payments in predefined increments tied to the completion of specific milestones.

- **Buyer Security**: Ensure funds are released only after the buyer's approval of a successfully completed milestone.

- **Dispute Resolution**: Provide a robust mechanism for handling disputes, including arbitration and fund recovery for buyers in case of incomplete deliveries.

## Features Implemented

### Payment Structure Setup

- **Flexible Milestone Definition**: Buyers can define the total transaction amount and specify multiple milestones, each with a corresponding percentage of the total payment.

- **Escrow Lock-in**: The entire agreed-upon payment is securely locked in an escrow held by the smart contract at the initiation of the transaction.

### Milestone Completion and Payment Release

- **Seller Submission**: Sellers can submit proof of completion for individual milestones.

- **Buyer Approval**: Buyers have the sole authority to approve the release of funds for completed milestones.

- **Unresponsive Buyer Handling**: Mechanisms are in place for automatic payment release after a defined timeout period if a buyer fails to respond to a milestone completion submission.

### Dispute Handling

- **Dispute Initiation**: Buyers can formally dispute a milestone's completion if they are not satisfied.

- **Resolution Process**: Disputes can be resolved through pre-defined conditions or an arbitration process.

- **Fund Recovery**: In cases where a dispute is resolved in the buyer's favor, funds for incomplete milestones can be refunded.

### Event Emission

The contract emits comprehensive events to provide transparency and traceability for all key actions:

- `init`: Contract initialization.

- `new_tx`: Creation of a new staggered payment transaction.

- `submit`: Milestone completion submitted by the seller.

- `approve`: Milestone approval by the buyer, leading to fund release.

- `dispute`: Initiation of a dispute for a milestone.

- `resolve`: Resolution of a dispute.

- `refund`: Refund of funds to the buyer.

- `close_tx`: Final closure of a transaction.

### Edge Case Handling

- **Milestone Integrity**: Strict controls prevent skipping or manipulating milestone order or status.

- **Party Unresponsiveness**: The contract includes logic to handle scenarios where either the buyer or seller becomes unresponsive, ensuring the transaction can still progress or or be resolved.

- **Fund Security**: Robust authorization checks and escrow mechanisms prevent unauthorized access or withdrawal of funds.

## How to Use (High-Level)

1.  **Deployment**: Deploy the `StaggeredPaymentContract` to the Stellar network using the Soroban CLI.

2.  **Initialization**: Call the `initialize` function, providing the address of the payment token to be used for transactions.

3.  **Create Transaction**: Buyers initiate a transaction by calling `create_transaction`, specifying the seller, total amount, milestone percentages, and descriptions. The total amount will be transferred to the contract's escrow.

4.  **Submit Milestone**: Once a milestone is complete, the seller calls `submit_milestone` with the transaction ID and milestone index.

5.  **Approve Milestone**: The buyer reviews the completed milestone and calls `approve_milestone` to release the corresponding funds to the seller.

6.  **Dispute (Optional)**: If the buyer is unsatisfied, they can call `dispute_milestone`. The contract will then follow the predefined dispute resolution process.

7.  **Finalize Transaction**: Once all milestones are completed and approved, the transaction can be formally closed.

## Testing

The project includes a comprehensive suite of tests to ensure contract reliability and correctness.

### Unit Testing

- **`test_create_transaction`**: Verifies the successful creation of a new transaction, including fund locking and event emission.

- **`test_event_emission`**: Specifically checks that the `new_tx` event is correctly emitted upon transaction creation.

- **`test_milestone_workflow`**: Validates the end-to-end process of milestone submission, approval, and fund release. This includes verifying the state changes of milestones and the emission of `submit` and `approve` events.

- **Timeout Scenarios**: (To be implemented/verified) Tests for automatic payment release if a buyer is unresponsive after a milestone submission.

- **Dispute Resolution Outcomes**: (To be implemented/verified) Tests for successful dispute initiation, resolution, and fund recovery flows.

### Integration Testing

- Simulate full transaction cycles within a test marketplace environment.

- Verify interaction between buyers, sellers, and the escrow contract in a multi-contract setup.
