use soroban_sdk::{Address, Env, String, Symbol, Vec, contracttype, symbol_short};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractStatus {
    Created,
    Funded,
    Delivered,
    Completed,
    Cancelled,
    RefundRequested,
    RefundProcessed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefundContract {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub token: Address,
    pub amount: u128,
    pub escrowed_amount: u128,
    pub status: ContractStatus,
    pub refund_deadline: u64,
    pub delivery_deadline: u64,
    pub refund_conditions: String,
    pub created_at: u64,
    pub funded_at: Option<u64>,
    pub delivered_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub cancelled_at: Option<u64>,
    pub refund_requested_at: Option<u64>,
    pub refund_processed_at: Option<u64>,
    pub refund_reason: Option<String>,
    pub refund_requester: Option<Address>,
}

// Storage key types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    Contract(u64),
    UserContracts(Address),
    ContractCounter,
}

// Storage key constants
const CONTRACT_COUNTER: Symbol = symbol_short!("CTR_CNT");

// Contract ID generation
pub fn get_next_contract_id(env: &Env) -> u64 {
    let current = env
        .storage()
        .instance()
        .get(&CONTRACT_COUNTER)
        .unwrap_or(0u64);
    let next = current + 1;
    env.storage().instance().set(&CONTRACT_COUNTER, &next);
    next
}

// Contract storage functions
pub fn get_contract(env: &Env, contract_id: u64) -> Option<RefundContract> {
    let key = StorageKey::Contract(contract_id);
    env.storage().persistent().get(&key)
}

pub fn set_contract(env: &Env, contract: &RefundContract) {
    let key = StorageKey::Contract(contract.id);
    env.storage().persistent().set(&key, contract);
}

// User contracts storage functions
pub fn get_user_contracts(env: &Env, user: &Address) -> Vec<u64> {
    let key = StorageKey::UserContracts(user.clone());
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(env))
}

pub fn add_user_contract(env: &Env, user: &Address, contract_id: u64) {
    let key = StorageKey::UserContracts(user.clone());
    let mut contracts = get_user_contracts(env, user);
    contracts.push_back(contract_id);
    env.storage().persistent().set(&key, &contracts);
}
