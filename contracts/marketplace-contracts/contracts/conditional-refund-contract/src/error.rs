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
    ContractExpired = 13,

    // Refund errors
    RefundNotRequested = 14,
    RefundAlreadyRequested = 15,
    RefundAlreadyProcessed = 16,
    RefundDeadlinePassed = 17,
    RefundConditionsNotMet = 18,
    DeliveryDeadlinePassed = 19,
    DeliveryAlreadyConfirmed = 20,
    DeliveryNotMarked = 21,

    // Payment errors
    InsufficientBalance = 22,
    TransferFailed = 23,
    InvalidAmount = 24,

    // General errors
    InvalidInput = 25,
    DataNotFound = 26,
    OperationNotAllowed = 27,
    DeadlineInPast = 28,
}
