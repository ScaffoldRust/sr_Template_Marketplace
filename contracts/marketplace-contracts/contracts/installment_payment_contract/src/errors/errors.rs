use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Clone, PartialEq)]
pub enum ContractError {
    AlreadyInstantiated = 1,
    InvalidAmount = 2,
    DuplicateUsers = 3,
    ArbitratorNotAllowed = 4,
    InvalidTimestamp = 5,
    InvalidAgreementId = 6,
    NotAuthorized = 7,
    AgreementNotFOund = 8,
}
