use crate::error::ContractError;
use crate::events::*;
use crate::escrow_storage;
use crate::escrow_storage::*;
use soroban_sdk::{token, Address, Env, String, Vec};

pub fn create_escrow(
    env: &Env,
    buyer: &Address,
    seller: &Address,
    arbitrator: &Address,
    token: &Address,
    amount: u128,
    description: String,
) -> Result<u64, ContractError> {
    if amount == 0 {
        return Err(ContractError::InvalidAmount);
    }

    if buyer == seller || buyer == arbitrator || seller == arbitrator {
        return Err(ContractError::InvalidInput);
    }

    let escrow_id = escrow_storage::get_next_escrow_id(env);
    let timestamp = env.ledger().timestamp();

    let escrow = Escrow {
        id: escrow_id,
        buyer: buyer.clone(),
        seller: seller.clone(),
        arbitrator: arbitrator.clone(),
        token: token.clone(),
        amount,
        description: description.clone(),
        status: EscrowStatus::Created,
        created_at: timestamp,
        funded_at: None,
        completed_at: None,
        disputed_at: None,
        dispute_reason: None,
    };

    set_escrow(env, &escrow);
    add_user_escrow(env, buyer, escrow_id);
    add_user_escrow(env, seller, escrow_id);
    add_user_escrow(env, arbitrator, escrow_id);

    emit_escrow_created(
        env,
        escrow_id,
        buyer.clone(),
        seller.clone(),
        arbitrator.clone(),
        token.clone(),
        amount,
        description,
    );

    Ok(escrow_id)
}

pub fn deposit(
    env: &Env,
    escrow_id: u64,
    buyer: &Address,
) -> Result<(), ContractError> {
    let mut escrow = get_escrow(env, escrow_id)?;

    if escrow.buyer != *buyer {
        return Err(ContractError::BuyerOnly);
    }

    if escrow.status != EscrowStatus::Created {
        return Err(ContractError::EscrowAlreadyFunded);
    }

    // Transfer tokens from buyer to contract
    let contract_address = env.current_contract_address();
    let token_client = token::Client::new(env, &escrow.token);
    
    token_client.transfer(buyer, &contract_address, &(escrow.amount as i128));

    escrow.status = EscrowStatus::Funded;
    escrow.funded_at = Some(env.ledger().timestamp());

    set_escrow(env, &escrow);

    emit_deposited(env, escrow_id, buyer.clone(), escrow.amount);

    Ok(())
}

pub fn release_funds(
    env: &Env,
    escrow_id: u64,
    buyer: &Address,
) -> Result<(), ContractError> {
    let mut escrow = get_escrow(env, escrow_id)?;

    if escrow.buyer != *buyer {
        return Err(ContractError::BuyerOnly);
    }

    if escrow.status != EscrowStatus::Funded {
        return Err(ContractError::EscrowNotFunded);
    }

    // Transfer tokens from contract to seller
    let contract_address = env.current_contract_address();
    let token_client = token::Client::new(env, &escrow.token);
    
    token_client.transfer(&contract_address, &escrow.seller, &(escrow.amount as i128));

    escrow.status = EscrowStatus::Completed;
    escrow.completed_at = Some(env.ledger().timestamp());

    set_escrow(env, &escrow);

    emit_funds_released(
        env,
        escrow_id,
        buyer.clone(),
        escrow.seller.clone(),
        escrow.amount,
    );

    Ok(())
}

pub fn raise_dispute(
    env: &Env,
    escrow_id: u64,
    disputer: &Address,
    reason: String,
) -> Result<(), ContractError> {
    let mut escrow = get_escrow(env, escrow_id)?;

    if escrow.buyer != *disputer && escrow.seller != *disputer {
        return Err(ContractError::ParticipantOnly);
    }

    if escrow.status != EscrowStatus::Funded {
        return Err(ContractError::EscrowNotFunded);
    }

    escrow.status = EscrowStatus::Disputed;
    escrow.disputed_at = Some(env.ledger().timestamp());
    escrow.dispute_reason = Some(reason.clone());

    set_escrow(env, &escrow);

    emit_dispute_raised(env, escrow_id, disputer.clone(), reason);

    Ok(())
}

pub fn arbitrate(
    env: &Env,
    escrow_id: u64,
    arbitrator: &Address,
    release_to_seller: bool,
) -> Result<(), ContractError> {
    let mut escrow = get_escrow(env, escrow_id)?;

    if escrow.arbitrator != *arbitrator {
        return Err(ContractError::ArbitratorOnly);
    }

    if escrow.status != EscrowStatus::Disputed {
        return Err(ContractError::EscrowNotDisputed);
    }

    let contract_address = env.current_contract_address();
    let token_client = token::Client::new(env, &escrow.token);

    if release_to_seller {
        // Release funds to seller
        token_client.transfer(&contract_address, &escrow.seller, &(escrow.amount as i128));
    } else {
        // Refund to buyer
        token_client.transfer(&contract_address, &escrow.buyer, &(escrow.amount as i128));
    }

    escrow.status = EscrowStatus::Completed;
    escrow.completed_at = Some(env.ledger().timestamp());

    set_escrow(env, &escrow);

    emit_arbitration_completed(
        env,
        escrow_id,
        arbitrator.clone(),
        release_to_seller,
        escrow.amount,
    );

    Ok(())
}

pub fn refund(
    env: &Env,
    escrow_id: u64,
    requester: &Address,
) -> Result<(), ContractError> {
    let mut escrow = get_escrow(env, escrow_id)?;

    // Only buyer or seller can request refund for non-disputed escrows
    if escrow.buyer != *requester && escrow.seller != *requester {
        return Err(ContractError::ParticipantOnly);
    }

    // Can only refund if escrow is funded but not disputed
    if escrow.status != EscrowStatus::Funded {
        return Err(ContractError::OperationNotAllowed);
    }

    // Transfer tokens back to buyer
    let contract_address = env.current_contract_address();
    let token_client = token::Client::new(env, &escrow.token);
    
    token_client.transfer(&contract_address, &escrow.buyer, &(escrow.amount as i128));

    escrow.status = EscrowStatus::Cancelled;
    escrow.completed_at = Some(env.ledger().timestamp());

    set_escrow(env, &escrow);

    emit_refunded(env, escrow_id, escrow.buyer.clone(), escrow.amount);

    Ok(())
}

pub fn get_escrow(env: &Env, escrow_id: u64) -> Result<Escrow, ContractError> {
    match escrow_storage::get_escrow(env, escrow_id) {
        Some(escrow) => Ok(escrow),
        None => Err(ContractError::EscrowNotFound),
    }
}

pub fn get_user_escrows(
    env: &Env,
    user: &Address,
    offset: u32,
    limit: u32,
) -> Result<Vec<u64>, ContractError> {
    let all_escrows = escrow_storage::get_user_escrows(env, user);
    let mut result = Vec::new(env);
    
    let start = offset as usize;
    let end = (offset + limit) as usize;
    let escrows_len = all_escrows.len() as usize;
    
    for i in start..end.min(escrows_len) {
        if let Some(escrow_id) = all_escrows.get(i as u32) {
            result.push_back(escrow_id);
        }
    }

    Ok(result)
}