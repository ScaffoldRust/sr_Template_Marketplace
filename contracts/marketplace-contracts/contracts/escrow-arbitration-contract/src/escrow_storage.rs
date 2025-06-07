use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Created,
    Funded,
    Completed,
    Disputed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub arbitrator: Address,
    pub token: Address,
    pub amount: u128,
    pub description: String,
    pub status: EscrowStatus,
    pub created_at: u64,
    pub funded_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub disputed_at: Option<u64>,
    pub dispute_reason: Option<String>,
}

// Storage key types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    Escrow(u64),
    UserEscrows(Address),
    EscrowCounter,
}

// Storage key constants
const ESCROW_COUNTER: Symbol = symbol_short!("ESC_CNT");

// Escrow ID generation
pub fn get_next_escrow_id(env: &Env) -> u64 {
    let current = env.storage().instance().get(&ESCROW_COUNTER).unwrap_or(0u64);
    let next = current + 1;
    env.storage().instance().set(&ESCROW_COUNTER, &next);
    next
}

// Escrow storage functions
pub fn get_escrow(env: &Env, escrow_id: u64) -> Option<Escrow> {
    let key = StorageKey::Escrow(escrow_id);
    env.storage().persistent().get(&key)
}

pub fn set_escrow(env: &Env, escrow: &Escrow) {
    let key = StorageKey::Escrow(escrow.id);
    env.storage().persistent().set(&key, escrow);
}

// User escrows storage functions
pub fn get_user_escrows(env: &Env, user: &Address) -> Vec<u64> {
    let key = StorageKey::UserEscrows(user.clone());
    env.storage().persistent().get(&key).unwrap_or(Vec::new(env))
}

pub fn add_user_escrow(env: &Env, user: &Address, escrow_id: u64) {
    let key = StorageKey::UserEscrows(user.clone());
    let mut escrows = get_user_escrows(env, user);
    escrows.push_back(escrow_id);
    env.storage().persistent().set(&key, &escrows);
}