use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractStatus {
    Created,
    Funded,
    Active,
    Completed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MilestoneStatus {
    Pending,
    Completed,
    Approved,
    Disputed,
    Resolved,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneData {
    pub description: String,
    pub amount: u128,
    pub release_criteria: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub id: u32,
    pub description: String,
    pub amount: u128,
    pub release_criteria: String,
    pub status: MilestoneStatus,
    pub completed_at: Option<u64>,
    pub approved_at: Option<u64>,
    pub dispute_reason: Option<String>,
    pub disputed_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contract {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub token: Address,
    pub total_amount: u128,
    pub escrowed_amount: u128,
    pub released_amount: u128,
    pub status: ContractStatus,
    pub created_at: u64,
    pub funded_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub cancelled_at: Option<u64>,
    pub milestone_count: u32,
}

// Storage key types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    Contract(u64),
    Milestone(u64, u32), // (contract_id, milestone_id)
    UserContracts(Address),
    ContractMilestones(u64),
    ContractCounter, // For generating unique contract IDs
}

// Storage key constants
const CONTRACT_COUNTER: Symbol = symbol_short!("CTR_CNT");

// Contract ID generation - using sequential counter
pub fn get_next_contract_id(env: &Env) -> u64 {
    let current = env.storage().instance().get(&CONTRACT_COUNTER).unwrap_or(0u64);
    let next = current + 1;
    env.storage().instance().set(&CONTRACT_COUNTER, &next);
    next
}

// Contract storage functions

pub fn get_contract(env: &Env, contract_id: u64) -> Option<Contract> {
    let key = StorageKey::Contract(contract_id);
    env.storage().persistent().get(&key)
}

pub fn set_contract(env: &Env, contract: &Contract) {
    let key = StorageKey::Contract(contract.id);
    env.storage().persistent().set(&key, contract);
}

// Milestone storage functions
pub fn get_milestone(env: &Env, contract_id: u64, milestone_id: u32) -> Option<Milestone> {
    let key = StorageKey::Milestone(contract_id, milestone_id);
    env.storage().persistent().get(&key)
}

pub fn set_milestone(env: &Env, contract_id: u64, milestone: &Milestone) {
    let key = StorageKey::Milestone(contract_id, milestone.id);
    env.storage().persistent().set(&key, milestone);
}

// User contracts storage functions
pub fn get_user_contracts(env: &Env, user: &Address) -> Vec<u64> {
    let key = StorageKey::UserContracts(user.clone());
    env.storage().persistent().get(&key).unwrap_or(Vec::new(env))
}

pub fn add_user_contract(env: &Env, user: &Address, contract_id: u64) {
    let key = StorageKey::UserContracts(user.clone());
    let mut contracts = get_user_contracts(env, user);
    contracts.push_back(contract_id);
    env.storage().persistent().set(&key, &contracts);
}

// Contract milestones list storage
pub fn get_contract_milestone_ids(env: &Env, contract_id: u64) -> Vec<u32> {
    let key = StorageKey::ContractMilestones(contract_id);
    env.storage().persistent().get(&key).unwrap_or(Vec::new(env))
}

pub fn set_contract_milestone_ids(env: &Env, contract_id: u64, milestone_ids: &Vec<u32>) {
    let key = StorageKey::ContractMilestones(contract_id);
    env.storage().persistent().set(&key, milestone_ids);
}
