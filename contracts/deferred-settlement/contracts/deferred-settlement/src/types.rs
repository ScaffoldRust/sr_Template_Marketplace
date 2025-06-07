use soroban_sdk::{contracttype, Address, Symbol};

#[contracttype]
pub enum DataKey {
    Transaction(u128),
    TotalTransactions,
    Admin,
    TokenContract,
}

#[contracttype]
#[derive(Clone)]
pub struct DeferredTransaction {
    pub id: u128,
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub condition: SettlementCondition,
    pub created_at: u64,
    pub deadline: u64,
    pub status: TransactionStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettlementCondition {
    TimeBased,
    BuyerApproval,
    OracleConfirmation,
}

#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Refunded,
    Disputed,
}

#[contracttype]
#[derive(Clone)]
pub enum TransactionEvent {
    TransactionCreated(u128, Address, Address, i128),
    ConditionVerified(u128, Symbol),
    FundsReleased(u128, Address),
    FundsRefunded(u128, Address),
    DisputeInitiated(u128),
    DisputeResolved(u128, bool),
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum Error {
    InvalidAmount,
    TransactionNotFound,
    NotPending,
    Unauthorized,
    ConditionNotMet,
    TokenContractNotSet,
}
