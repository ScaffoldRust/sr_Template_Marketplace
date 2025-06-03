use soroban_sdk::{contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCreatedEvent {
    pub contract_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub token: Address,
    pub total_amount: u128,
    pub milestone_count: u32,
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
pub struct MilestoneCompletedEvent {
    pub contract_id: u64,
    pub milestone_id: u32,
    pub completor: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneApprovedEvent {
    pub contract_id: u64,
    pub milestone_id: u32,
    pub buyer: Address,
    pub amount_released: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneDisputedEvent {
    pub contract_id: u64,
    pub milestone_id: u32,
    pub disputer: Address,
    pub reason: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeResolvedEvent {
    pub contract_id: u64,
    pub milestone_id: u32,
    pub admin: Address,
    pub approved: bool,
    pub amount_released: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCancelledEvent {
    pub contract_id: u64,
    pub canceller: Address,
    pub refunded_amount: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCompletedEvent {
    pub contract_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub total_paid: u128,
}

pub fn emit_contract_created(
    env: &Env,
    contract_id: u64,
    buyer: Address,
    seller: Address,
    token: Address,
    total_amount: u128,
    milestone_count: u32,
) {
    let event = ContractCreatedEvent {
        contract_id,
        buyer,
        seller,
        token,
        total_amount,
        milestone_count,
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

pub fn emit_milestone_completed(
    env: &Env,
    contract_id: u64,
    milestone_id: u32,
    completor: Address,
) {
    let event = MilestoneCompletedEvent {
        contract_id,
        milestone_id,
        completor,
    };
    env.events().publish(("milestone_completed",), event);
}

pub fn emit_milestone_approved(
    env: &Env,
    contract_id: u64,
    milestone_id: u32,
    buyer: Address,
    amount_released: u128,
) {
    let event = MilestoneApprovedEvent {
        contract_id,
        milestone_id,
        buyer,
        amount_released,
    };
    env.events().publish(("milestone_approved",), event);
}

pub fn emit_milestone_disputed(
    env: &Env,
    contract_id: u64,
    milestone_id: u32,
    disputer: Address,
    reason: String,
) {
    let event = MilestoneDisputedEvent {
        contract_id,
        milestone_id,
        disputer,
        reason,
    };
    env.events().publish(("milestone_disputed",), event);
}

pub fn emit_dispute_resolved(
    env: &Env,
    contract_id: u64,
    milestone_id: u32,
    admin: Address,
    approved: bool,
    amount_released: u128,
) {
    let event = DisputeResolvedEvent {
        contract_id,
        milestone_id,
        admin,
        approved,
        amount_released,
    };
    env.events().publish(("dispute_resolved",), event);
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

pub fn emit_contract_completed(
    env: &Env,
    contract_id: u64,
    buyer: Address,
    seller: Address,
    total_paid: u128,
) {
    let event = ContractCompletedEvent {
        contract_id,
        buyer,
        seller,
        total_paid,
    };
    env.events().publish(("contract_completed",), event);
}