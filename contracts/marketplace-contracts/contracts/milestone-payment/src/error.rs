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
    ParticipantOnly = 7,
    
    // Contract errors
    ContractNotFound = 8,
    ContractAlreadyFunded = 9,
    ContractNotFunded = 10,
    ContractCancelled = 11,
    ContractCompleted = 12,
    
    // Milestone errors
    MilestoneNotFound = 13,
    MilestoneAlreadyCompleted = 14,
    MilestoneNotCompleted = 15,
    MilestoneDisputed = 16,
    MilestoneNotDisputed = 17,
    InvalidMilestoneData = 18,
    
    // Payment errors
    InsufficientBalance = 19,
    TransferFailed = 20,
    InvalidAmount = 21,
    TotalAmountMismatch = 22,
    
    // General errors
    InvalidInput = 23,
    DataNotFound = 24,
    OperationNotAllowed = 25,
}