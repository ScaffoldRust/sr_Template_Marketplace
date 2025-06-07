use crate::types::{DataKey, DeferredTransaction, Error};
use soroban_sdk::Env;

pub fn get_transaction(env: Env, transaction_id: u128) -> Result<DeferredTransaction, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Transaction(transaction_id))
        .ok_or(Error::TransactionNotFound)
}

pub fn get_total_transactions(env: Env) -> u128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalTransactions)
        .unwrap_or(0)
}
