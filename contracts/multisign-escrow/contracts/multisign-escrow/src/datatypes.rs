use core::fmt;
use soroban_sdk::{contracterror, contracttype, Address, Map};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Initialized,
    Deposited,
    Released,
    Refunded,
}

#[contracttype]
#[derive(Clone)]
pub enum Datakey {
    EscrowState,
}

#[contracttype]
#[derive(Clone)]
pub struct EscrowState {
    pub buyer: Address,
    pub seller: Address,
    pub mediator: Option<Address>,
    pub asset: Address,
    pub amount: i128,
    pub approvals: Map<Address, bool>,
    pub approved_count: u32,
    pub required_approvals: u32,
    pub status: EscrowStatus,
    pub deadline: u64,
}

#[contracterror]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EscrowError {
    Unauthorized = 1,
    InvalidAmount = 2,
    InvalidRequiredApprovals = 3,
    InvalidStatus = 4,
    AlreadyApproved = 5,
    NotEnoughApprovals = 6,
    DeadlineNotReached = 7,
    AlreadyInitialized = 8,
}

impl fmt::Display for EscrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EscrowError::Unauthorized => write!(f, "Unauthorized operation"),
            EscrowError::InvalidAmount => write!(f, "Input data is invalid"),
            EscrowError::InvalidRequiredApprovals => write!(f, "Requested resource not found"),
            EscrowError::InvalidStatus => write!(f, "This operation is not allowed"),
            EscrowError::AlreadyApproved => write!(f, "Arithmetic operation failed"),
            EscrowError::NotEnoughApprovals => write!(f, "Already initialized"),
            EscrowError::DeadlineNotReached => write!(f, "Dead line not reached"),
            EscrowError::AlreadyInitialized => write!(f, "already initilized"),
        }
    }
}
