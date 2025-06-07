#![no_std]

mod error;
mod events;
mod storage;
mod escrow_storage;
mod contract;
mod test;

use soroban_sdk::{
    contract, contractimpl, Address, Env, String,
};

pub use error::*;
pub use events::*;
pub use contract::*;

#[contract]
pub struct EscrowArbitrationContract;

#[contractimpl]
impl EscrowArbitrationContract {
    /// Initialize the contract with admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if storage::has_admin(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        Ok(())
    }

    /// Create a new escrow transaction
    pub fn create_escrow(
        env: Env,
        buyer: Address,
        seller: Address,
        arbitrator: Address,
        token: Address,
        amount: u128,
        description: String,
    ) -> Result<u64, ContractError> {
        buyer.require_auth();
        contract::create_escrow(&env, &buyer, &seller, &arbitrator, &token, amount, description)
    }

    /// Deposit funds into escrow
    pub fn deposit(
        env: Env,
        escrow_id: u64,
        buyer: Address,
    ) -> Result<(), ContractError> {
        buyer.require_auth();
        contract::deposit(&env, escrow_id, &buyer)
    }

    /// Release funds to seller (standard release)
    pub fn release_funds(
        env: Env,
        escrow_id: u64,
        buyer: Address,
    ) -> Result<(), ContractError> {
        buyer.require_auth();
        contract::release_funds(&env, escrow_id, &buyer)
    }

    /// Raise a dispute
    pub fn raise_dispute(
        env: Env,
        escrow_id: u64,
        disputer: Address,
        reason: String,
    ) -> Result<(), ContractError> {
        disputer.require_auth();
        contract::raise_dispute(&env, escrow_id, &disputer, reason)
    }

    /// Arbitrate dispute (arbitrator only)
    pub fn arbitrate(
        env: Env,
        escrow_id: u64,
        arbitrator: Address,
        release_to_seller: bool,
    ) -> Result<(), ContractError> {
        arbitrator.require_auth();
        contract::arbitrate(&env, escrow_id, &arbitrator, release_to_seller)
    }

    /// Refund funds to buyer
    pub fn refund(
        env: Env,
        escrow_id: u64,
        requester: Address,
    ) -> Result<(), ContractError> {
        requester.require_auth();
        contract::refund(&env, escrow_id, &requester)
    }

    /// Get escrow details
    pub fn get_escrow(
        env: Env,
        escrow_id: u64,
    ) -> Result<escrow_storage::Escrow, ContractError> {
        contract::get_escrow(&env, escrow_id)
    }

    /// Get escrows for a user
    pub fn get_user_escrows(
        env: Env,
        user: Address,
        offset: u32,
        limit: u32,
    ) -> Result<soroban_sdk::Vec<u64>, ContractError> {
        contract::get_user_escrows(&env, &user, offset, limit)
    }
}