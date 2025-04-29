use soroban_sdk::{contracttype, Address};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const TRANSACTION_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const TRANSACTION_LIFETIME_THRESHOLD: u32 = TRANSACTION_BUMP_AMOUNT - DAY_IN_LEDGERS;

// Status of a cancellation proposal
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum CancellationStatus {
    None,                 // No cancellation proposal exists
    ProposedByBuyer,      // Buyer has proposed cancellation
    ProposedBySeller,     // Seller has proposed cancellation
    Completed,            // Cancellation completed and funds returned
}

#[derive(Clone)]
#[contracttype]
pub struct Transaction {
    pub id: u64,                   // Unique transaction ID
    pub buyer: Address,            // Buyer's address
    pub seller: Address,           // Seller's address
    pub token: Address,            // Token contract address 
    pub amount: i128,              // Amount of tokens in escrow
    pub status: CancellationStatus, // Current status of cancellation
    pub proposal_timestamp: u64,   // Timestamp when cancellation was proposed
    pub response_window: u64,      // Time window for the counterparty to respond
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TransactionCounter,           // Counter for generating unique transaction IDs
    Transaction(u64),             // Transaction data by ID
    BuyerTransactions(Address),   // List of transaction IDs for a buyer
    SellerTransactions(Address),  // List of transaction IDs for a seller
    ResponseWindow,               // Default time window for responding to cancellation requests
} 