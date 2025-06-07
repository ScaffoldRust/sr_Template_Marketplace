#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

mod admin;
mod dispute;
mod query;
mod settlement;
mod transaction;
mod types;

#[cfg(test)]
mod test;

use types::{DataKey, DeferredTransaction, SettlementCondition};

#[contract]
pub struct DeferredSettlementContract;

#[contractimpl]
impl DeferredSettlementContract {
    pub fn initialize(env: Env, admin: Address) {
        admin::initialize(env, admin);
    }

    pub fn set_token_contract(env: Env, admin: Address, token_contract: Address) {
        admin.require_auth();
        if !admin::is_admin(&env, &admin) {
            panic!("Unauthorized");
        }
        env.storage()
            .instance()
            .set(&DataKey::TokenContract, &token_contract);
    }

    pub fn create_transaction(
        env: Env,
        buyer: Address,
        seller: Address,
        amount: i128,
        condition: Symbol,
        duration: u64,
    ) -> u128 {
        let condition = match condition {
            condition if condition == Symbol::new(&env, "TimeBased") => {
                SettlementCondition::TimeBased
            }
            condition if condition == Symbol::new(&env, "BuyerApproval") => {
                SettlementCondition::BuyerApproval
            }
            condition if condition == Symbol::new(&env, "OracleConfirmation") => {
                SettlementCondition::OracleConfirmation
            }
            _ => panic!("Invalid condition"),
        };
        transaction::create_transaction(env, buyer, seller, amount, condition, duration).unwrap()
    }

    pub fn verify_condition(
        env: Env,
        caller: Address,
        transaction_id: u128,
        oracle_input: Option<bool>,
    ) {
        settlement::verify_condition(env, caller, transaction_id, oracle_input).unwrap()
    }

    pub fn initiate_dispute(env: Env, caller: Address, transaction_id: u128) {
        dispute::initiate_dispute(env, caller, transaction_id).unwrap()
    }

    pub fn resolve_dispute(
        env: Env,
        caller: Address,
        transaction_id: u128,
        release_to_seller: bool,
    ) {
        dispute::resolve_dispute(env, caller, transaction_id, release_to_seller).unwrap()
    }

    pub fn get_transaction(env: Env, transaction_id: u128) -> DeferredTransaction {
        query::get_transaction(env, transaction_id).unwrap()
    }

    pub fn get_total_transactions(env: Env) -> u128 {
        query::get_total_transactions(env)
    }
}
