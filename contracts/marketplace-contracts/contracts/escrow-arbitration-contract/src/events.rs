use soroban_sdk::{contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowCreatedEvent {
    pub escrow_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub arbitrator: Address,
    pub token: Address,
    pub amount: u128,
    pub description: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositedEvent {
    pub escrow_id: u64,
    pub buyer: Address,
    pub amount: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundsReleasedEvent {
    pub escrow_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub amount: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeRaisedEvent {
    pub escrow_id: u64,
    pub disputer: Address,
    pub reason: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArbitrationCompletedEvent {
    pub escrow_id: u64,
    pub arbitrator: Address,
    pub release_to_seller: bool,
    pub amount: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefundedEvent {
    pub escrow_id: u64,
    pub buyer: Address,
    pub amount: u128,
}

pub fn emit_escrow_created(
    env: &Env,
    escrow_id: u64,
    buyer: Address,
    seller: Address,
    arbitrator: Address,
    token: Address,
    amount: u128,
    description: String,
) {
    let event = EscrowCreatedEvent {
        escrow_id,
        buyer,
        seller,
        arbitrator,
        token,
        amount,
        description,
    };
    env.events().publish(("escrow_created",), event);
}

pub fn emit_deposited(env: &Env, escrow_id: u64, buyer: Address, amount: u128) {
    let event = DepositedEvent {
        escrow_id,
        buyer,
        amount,
    };
    env.events().publish(("deposited",), event);
}

pub fn emit_funds_released(
    env: &Env,
    escrow_id: u64,
    buyer: Address,
    seller: Address,
    amount: u128,
) {
    let event = FundsReleasedEvent {
        escrow_id,
        buyer,
        seller,
        amount,
    };
    env.events().publish(("funds_released",), event);
}

pub fn emit_dispute_raised(
    env: &Env,
    escrow_id: u64,
    disputer: Address,
    reason: String,
) {
    let event = DisputeRaisedEvent {
        escrow_id,
        disputer,
        reason,
    };
    env.events().publish(("dispute_raised",), event);
}

pub fn emit_arbitration_completed(
    env: &Env,
    escrow_id: u64,
    arbitrator: Address,
    release_to_seller: bool,
    amount: u128,
) {
    let event = ArbitrationCompletedEvent {
        escrow_id,
        arbitrator,
        release_to_seller,
        amount,
    };
    env.events().publish(("arbitration_completed",), event);
}

pub fn emit_refunded(env: &Env, escrow_id: u64, buyer: Address, amount: u128) {
    let event = RefundedEvent {
        escrow_id,
        buyer,
        amount,
    };
    env.events().publish(("refunded",), event);
}