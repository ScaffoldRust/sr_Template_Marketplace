#![no_std]

mod contract;
mod error;
mod events;
mod refund_storage;
mod storage;
mod test;

use soroban_sdk::{Address, Env, String, contract, contractimpl};

pub use contract::*;
pub use error::*;
pub use events::*;

#[contract]
pub struct ConditionalRefundContract;

#[contractimpl]
impl ConditionalRefundContract {
    /// Initialize the contract with admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if storage::has_admin(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        Ok(())
    }

    /// Create a new refund contract
    pub fn create_refund_contract(
        env: Env,
        buyer: Address,
        seller: Address,
        token: Address,
        amount: u128,
        refund_deadline: u64,
        delivery_deadline: u64,
        refund_conditions: String,
    ) -> Result<u64, ContractError> {
        buyer.require_auth();
        contract::create_refund_contract(
            &env,
            &buyer,
            &seller,
            &token,
            amount,
            refund_deadline,
            delivery_deadline,
            refund_conditions,
        )
    }

    /// Fund the contract by locking tokens in escrow
    pub fn fund_contract(env: Env, contract_id: u64, buyer: Address) -> Result<(), ContractError> {
        buyer.require_auth();
        contract::fund_contract(&env, contract_id, &buyer)
    }

    /// Mark order as delivered (seller action)
    pub fn mark_delivered(
        env: Env,
        contract_id: u64,
        seller: Address,
    ) -> Result<(), ContractError> {
        seller.require_auth();
        contract::mark_delivered(&env, contract_id, &seller)
    }

    /// Confirm delivery and release funds (buyer action)
    pub fn confirm_delivery(
        env: Env,
        contract_id: u64,
        buyer: Address,
    ) -> Result<(), ContractError> {
        buyer.require_auth();
        contract::confirm_delivery(&env, contract_id, &buyer)
    }

    /// Request refund based on conditions
    pub fn request_refund(
        env: Env,
        contract_id: u64,
        requester: Address,
        reason: String,
    ) -> Result<(), ContractError> {
        requester.require_auth();
        contract::request_refund(&env, contract_id, &requester, reason)
    }

    /// Process automatic refund if conditions are met
    pub fn process_automatic_refund(env: Env, contract_id: u64) -> Result<(), ContractError> {
        contract::process_automatic_refund(&env, contract_id)
    }

    /// Resolve refund dispute (admin only)
    pub fn resolve_refund_dispute(
        env: Env,
        contract_id: u64,
        admin: Address,
        approve_refund: bool,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        contract::resolve_refund_dispute(&env, contract_id, &admin, approve_refund)
    }

    /// Cancel contract (before delivery)
    pub fn cancel_contract(
        env: Env,
        contract_id: u64,
        canceller: Address,
    ) -> Result<(), ContractError> {
        canceller.require_auth();
        contract::cancel_contract(&env, contract_id, &canceller)
    }

    /// Get contract details
    pub fn get_contract(
        env: Env,
        contract_id: u64,
    ) -> Result<refund_storage::RefundContract, ContractError> {
        contract::get_contract(&env, contract_id)
    }

    /// Get user contracts
    pub fn get_user_contracts(
        env: Env,
        user: Address,
        offset: u32,
        limit: u32,
    ) -> Result<soroban_sdk::Vec<u64>, ContractError> {
        contract::get_user_contracts(&env, &user, offset, limit)
    }
}
