use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    // Initialization errors
    AlreadyInitialized = 1,
    NotInitialized = 2,
    
    // Authorization errors
    Unauthorized = 3,
    AdminOnly = 4,
    BuyerOnly = 5,
    SellerOnly = 6,
    ArbitratorOnly = 7,
    ParticipantOnly = 8,
    
    // Escrow errors
    EscrowNotFound = 9,
    EscrowAlreadyFunded = 10,
    EscrowNotFunded = 11,
    EscrowCompleted = 12,
    EscrowCancelled = 13,
    EscrowDisputed = 14,
    EscrowNotDisputed = 15,
    
    // Payment errors
    InvalidAmount = 16,
    InsufficientBalance = 17,
    TransferFailed = 18,
    
    // General errors
    InvalidInput = 19,
    DataNotFound = 20,
    OperationNotAllowed = 21,
}