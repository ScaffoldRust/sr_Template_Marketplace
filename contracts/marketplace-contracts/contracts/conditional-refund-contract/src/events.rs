use soroban_sdk::{Address, Env, String, contracttype};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCreatedEvent {
    pub contract_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub token: Address,
    pub amount: u128,
    pub refund_deadline: u64,
    pub delivery_deadline: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractFundedEvent {
    pub contract_id: u64,
    pub buyer: Address,
    pub amount: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryMarkedEvent {
    pub contract_id: u64,
    pub seller: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryConfirmedEvent {
    pub contract_id: u64,
    pub buyer: Address,
    pub amount_released: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefundRequestedEvent {
    pub contract_id: u64,
    pub requester: Address,
    pub reason: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefundProcessedEvent {
    pub contract_id: u64,
    pub recipient: Address,
    pub amount: u128,
    pub automatic: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefundDisputeResolvedEvent {
    pub contract_id: u64,
    pub admin: Address,
    pub approved: bool,
    pub amount: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCancelledEvent {
    pub contract_id: u64,
    pub canceller: Address,
    pub refunded_amount: u128,
}

// Event emission functions
pub fn emit_contract_created(
    env: &Env,
    contract_id: u64,
    buyer: Address,
    seller: Address,
    token: Address,
    amount: u128,
    refund_deadline: u64,
    delivery_deadline: u64,
) {
    let event = ContractCreatedEvent {
        contract_id,
        buyer,
        seller,
        token,
        amount,
        refund_deadline,
        delivery_deadline,
    };
    env.events().publish(("contract_created",), event);
}

pub fn emit_contract_funded(env: &Env, contract_id: u64, buyer: Address, amount: u128) {
    let event = ContractFundedEvent {
        contract_id,
        buyer,
        amount,
    };
    env.events().publish(("contract_funded",), event);
}

pub fn emit_delivery_marked(env: &Env, contract_id: u64, seller: Address) {
    let event = DeliveryMarkedEvent {
        contract_id,
        seller,
    };
    env.events().publish(("delivery_marked",), event);
}

pub fn emit_delivery_confirmed(env: &Env, contract_id: u64, buyer: Address, amount_released: u128) {
    let event = DeliveryConfirmedEvent {
        contract_id,
        buyer,
        amount_released,
    };
    env.events().publish(("delivery_confirmed",), event);
}

pub fn emit_refund_requested(env: &Env, contract_id: u64, requester: Address, reason: String) {
    let event = RefundRequestedEvent {
        contract_id,
        requester,
        reason,
    };
    env.events().publish(("refund_requested",), event);
}

pub fn emit_refund_processed(
    env: &Env,
    contract_id: u64,
    recipient: Address,
    amount: u128,
    automatic: bool,
) {
    let event = RefundProcessedEvent {
        contract_id,
        recipient,
        amount,
        automatic,
    };
    env.events().publish(("refund_processed",), event);
}

pub fn emit_refund_dispute_resolved(
    env: &Env,
    contract_id: u64,
    admin: Address,
    approved: bool,
    amount: u128,
) {
    let event = RefundDisputeResolvedEvent {
        contract_id,
        admin,
        approved,
        amount,
    };
    env.events().publish(("refund_dispute_resolved",), event);
}

pub fn emit_contract_cancelled(
    env: &Env,
    contract_id: u64,
    canceller: Address,
    refunded_amount: u128,
) {
    let event = ContractCancelledEvent {
        contract_id,
        canceller,
        refunded_amount,
    };
    env.events().publish(("contract_cancelled",), event);
}
