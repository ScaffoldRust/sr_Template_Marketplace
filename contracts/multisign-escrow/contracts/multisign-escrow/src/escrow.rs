use crate::datatypes::{Datakey, EscrowError, EscrowState, EscrowStatus};
use soroban_sdk::{
    contract, contractimpl,
    token::{self},
    Address, Env, Map, Symbol,
};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initializes the escrow contract.

    pub fn initialize(
        env: Env,
        buyer: Address,
        seller: Address,
        mediator: Option<Address>,
        asset: Address,
        amount: i128,
        required_approvals: u32,
        deadline: u64,
    ) -> Result<(), EscrowError> {
        if env.storage().instance().has(&Datakey::EscrowState) {
            return Err(EscrowError::AlreadyInitialized);
        }

        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        let mut max_possible_approvals = 2;
        if mediator.is_some() {
            max_possible_approvals = 3;
        }

        if required_approvals == 0 || required_approvals > max_possible_approvals {
            return Err(EscrowError::InvalidRequiredApprovals);
        }

        let state = EscrowState {
            buyer,
            seller,
            mediator,
            asset,
            amount,
            approvals: Map::new(&env),
            approved_count: 0,
            required_approvals,
            status: EscrowStatus::Initialized,
            deadline,
        };

        env.storage().instance().set(&Datakey::EscrowState, &state);
        Ok(())
    }

    /// Allows the buyer to deposit funds into the escrow.

    pub fn deposit(env: Env) -> Result<(), EscrowError> {
        let mut state: EscrowState = env
            .storage()
            .instance()
            .get(&Datakey::EscrowState)
            .ok_or(EscrowError::InvalidStatus)?;

        if state.status != EscrowStatus::Initialized {
            return Err(EscrowError::InvalidStatus);
        }

        state.buyer.require_auth();

        let token_client = token::Client::new(&env, &state.asset);
        token_client.transfer(&state.buyer, &env.current_contract_address(), &state.amount);

        state.status = EscrowStatus::Deposited;
        env.storage().instance().set(&Datakey::EscrowState, &state);

        env.events().publish(
            (Symbol::new(&env, "deposit"), state.buyer.clone()),
            state.amount,
        );

        Ok(())
    }

    pub fn approve(env: Env, party: Address) -> Result<(), EscrowError> {
        party.require_auth();

        let mut state: EscrowState = env
            .storage()
            .instance()
            .get(&Datakey::EscrowState)
            .ok_or(EscrowError::InvalidStatus)?;

        if state.status != EscrowStatus::Deposited {
            return Err(EscrowError::InvalidStatus);
        }

        let is_authorized = party == state.buyer
            || party == state.seller
            || (state.mediator.is_some() && party == state.mediator.clone().unwrap());

        if !is_authorized {
            return Err(EscrowError::Unauthorized);
        }

        if state.approvals.contains_key(party.clone()) {
            return Err(EscrowError::AlreadyApproved);
        }

        state.approvals.set(party.clone(), true);
        state.approved_count += 1;

        env.storage().instance().set(&Datakey::EscrowState, &state);

        env.events().publish(
            (Symbol::new(&env, "approval"), party.clone()),
            state.approved_count,
        );

        Ok(())
    }

    /// Releases funds to the seller once the required number of approvals are met.

    pub fn release(env: Env) -> Result<(), EscrowError> {
        let mut state: EscrowState = env
            .storage()
            .instance()
            .get(&Datakey::EscrowState)
            .ok_or(EscrowError::InvalidStatus)?;

        if state.status != EscrowStatus::Deposited {
            return Err(EscrowError::InvalidStatus);
        }

        if state.approved_count < state.required_approvals {
            return Err(EscrowError::NotEnoughApprovals);
        }

        state.status = EscrowStatus::Released;
        env.storage().instance().set(&Datakey::EscrowState, &state);

        let token_client = token::Client::new(&env, &state.asset);
        token_client.transfer(
            &env.current_contract_address(),
            &state.seller,
            &state.amount,
        );

        env.events().publish(
            (Symbol::new(&env, "release"), state.seller.clone()),
            state.amount,
        );

        Ok(())
    }

    pub fn refund(env: Env) -> Result<(), EscrowError> {
        let mut state: EscrowState = env
            .storage()
            .instance()
            .get(&Datakey::EscrowState)
            .ok_or(EscrowError::InvalidStatus)?;

        if state.status != EscrowStatus::Deposited {
            return Err(EscrowError::InvalidStatus);
        }

        let current_timestamp = env.ledger().timestamp();

        if current_timestamp < state.deadline && state.approved_count < state.required_approvals {
            return Err(EscrowError::DeadlineNotReached);
        }

        state.status = EscrowStatus::Refunded; // Change status before transfer
        env.storage().instance().set(&Datakey::EscrowState, &state);

        let token_client = token::Client::new(&env, &state.asset);
        token_client.transfer(&env.current_contract_address(), &state.buyer, &state.amount);

        env.events().publish(
            (Symbol::new(&env, "refund"), state.buyer.clone()),
            state.amount,
        );

        Ok(())
    }

    /// Gets the current state of the escrow.

    pub fn get_state(env: Env) -> Result<EscrowState, EscrowError> {
        env.storage()
            .instance()
            .get(&Datakey::EscrowState)
            .ok_or(EscrowError::InvalidStatus)
    }
}
