#![no_std]

mod error;
mod events;
mod storage;
mod milestone_storage;
mod contract;
mod test;

use soroban_sdk::{
    contract, contractimpl, Address, Env, Vec,
};

pub use error::*;
pub use events::*;
pub use contract::*;

#[contract]
pub struct MilestonePaymentContract;

#[contractimpl]
impl MilestonePaymentContract {
    /// Initialize the contract with admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if storage::has_admin(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        Ok(())
    }

    /// Create a new milestone-based payment contract
    pub fn create_contract(
        env: Env,
        buyer: Address,
        seller: Address,
        token: Address,
        total_amount: u128,
        milestones: Vec<milestone_storage::MilestoneData>,
    ) -> Result<u64, ContractError> {
        buyer.require_auth();
        contract::create_contract(&env, &buyer, &seller, &token, total_amount, milestones)
    }

    /// Fund the contract by locking tokens in escrow
    pub fn fund_contract(
        env: Env,
        contract_id: u64,
        buyer: Address,
    ) -> Result<(), ContractError> {
        buyer.require_auth();
        contract::fund_contract(&env, contract_id, &buyer)
    }

    /// Complete a milestone and release funds
    pub fn complete_milestone(
        env: Env,
        contract_id: u64,
        milestone_id: u32,
        completor: Address,
    ) -> Result<(), ContractError> {
        completor.require_auth();
        contract::complete_milestone(&env, contract_id, milestone_id, &completor)
    }

    /// Approve milestone completion (buyer approval)
    pub fn approve_milestone(
        env: Env,
        contract_id: u64,
        milestone_id: u32,
        buyer: Address,
    ) -> Result<(), ContractError> {
        buyer.require_auth();
        contract::approve_milestone(&env, contract_id, milestone_id, &buyer)
    }

    /// Dispute a milestone
    pub fn dispute_milestone(
        env: Env,
        contract_id: u64,
        milestone_id: u32,
        disputer: Address,
        reason: soroban_sdk::String,
    ) -> Result<(), ContractError> {
        disputer.require_auth();
        contract::dispute_milestone(&env, contract_id, milestone_id, &disputer, reason)
    }

    /// Resolve dispute (admin only)
    pub fn resolve_dispute(
        env: Env,
        contract_id: u64,
        milestone_id: u32,
        admin: Address,
        approve: bool,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        contract::resolve_dispute(&env, contract_id, milestone_id, &admin, approve)
    }

    /// Cancel contract and refund remaining funds
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
    ) -> Result<milestone_storage::Contract, ContractError> {
        contract::get_contract(&env, contract_id)
    }

    /// Get milestone details
    pub fn get_milestone(
        env: Env,
        contract_id: u64,
        milestone_id: u32,
    ) -> Result<milestone_storage::Milestone, ContractError> {
        contract::get_milestone(&env, contract_id, milestone_id)
    }

    /// Get all milestones for a contract
    pub fn get_contract_milestones(
        env: Env,
        contract_id: u64,
    ) -> Result<Vec<milestone_storage::Milestone>, ContractError> {
        contract::get_contract_milestones(&env, contract_id)
    }

    /// Get contracts for a user (buyer or seller)
    pub fn get_user_contracts(
        env: Env,
        user: Address,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<u64>, ContractError> {
        contract::get_user_contracts(&env, &user, offset, limit)
    }
}