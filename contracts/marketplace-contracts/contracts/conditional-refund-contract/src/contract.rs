use crate::error::ContractError;
use crate::events::*;
use crate::refund_storage::*;
use crate::storage;
use soroban_sdk::{Address, Env, String, Vec, token};

pub fn create_refund_contract(
    env: &Env,
    buyer: &Address,
    seller: &Address,
    token: &Address,
    amount: u128,
    refund_deadline: u64,
    delivery_deadline: u64,
    refund_conditions: String,
) -> Result<u64, ContractError> {
    if amount == 0 {
        return Err(ContractError::InvalidAmount);
    }

    if buyer == seller {
        return Err(ContractError::InvalidInput);
    }

    let current_time = env.ledger().timestamp();
    if refund_deadline <= current_time || delivery_deadline <= current_time {
        return Err(ContractError::DeadlineInPast);
    }

    let contract_id = get_next_contract_id(env);

    let contract = RefundContract {
        id: contract_id,
        buyer: buyer.clone(),
        seller: seller.clone(),
        token: token.clone(),
        amount,
        escrowed_amount: 0,
        status: ContractStatus::Created,
        refund_deadline,
        delivery_deadline,
        refund_conditions,
        created_at: current_time,
        funded_at: None,
        delivered_at: None,
        completed_at: None,
        cancelled_at: None,
        refund_requested_at: None,
        refund_processed_at: None,
        refund_reason: None,
        refund_requester: None,
    };

    set_contract(env, &contract);
    add_user_contract(env, buyer, contract_id);
    add_user_contract(env, seller, contract_id);

    emit_contract_created(
        env,
        contract_id,
        buyer.clone(),
        seller.clone(),
        token.clone(),
        amount,
        refund_deadline,
        delivery_deadline,
    );

    Ok(contract_id)
}

pub fn fund_contract(env: &Env, contract_id: u64, buyer: &Address) -> Result<(), ContractError> {
    let mut contract = get_contract(env, contract_id)?;

    if contract.buyer != *buyer {
        return Err(ContractError::BuyerOnly);
    }

    if contract.status != ContractStatus::Created {
        return Err(ContractError::ContractAlreadyFunded);
    }

    // Transfer tokens from buyer to contract
    let contract_address = env.current_contract_address();
    let token_client = token::Client::new(env, &contract.token);

    token_client.transfer(buyer, &contract_address, &(contract.amount as i128));

    contract.escrowed_amount = contract.amount;
    contract.status = ContractStatus::Funded;
    contract.funded_at = Some(env.ledger().timestamp());

    set_contract(env, &contract);

    emit_contract_funded(env, contract_id, buyer.clone(), contract.amount);

    Ok(())
}

pub fn mark_delivered(env: &Env, contract_id: u64, seller: &Address) -> Result<(), ContractError> {
    let mut contract = get_contract(env, contract_id)?;

    if contract.seller != *seller {
        return Err(ContractError::SellerOnly);
    }

    if contract.status != ContractStatus::Funded {
        return Err(ContractError::ContractNotFunded);
    }

    let current_time = env.ledger().timestamp();
    if current_time > contract.delivery_deadline {
        return Err(ContractError::DeliveryDeadlinePassed);
    }

    contract.status = ContractStatus::Delivered;
    contract.delivered_at = Some(current_time);

    set_contract(env, &contract);

    emit_delivery_marked(env, contract_id, seller.clone());

    Ok(())
}

pub fn confirm_delivery(env: &Env, contract_id: u64, buyer: &Address) -> Result<(), ContractError> {
    let mut contract = get_contract(env, contract_id)?;

    if contract.buyer != *buyer {
        return Err(ContractError::BuyerOnly);
    }

    if contract.status != ContractStatus::Delivered {
        return Err(ContractError::DeliveryNotMarked);
    }

    // Release funds to seller
    let contract_address = env.current_contract_address();
    let token_client = token::Client::new(env, &contract.token);

    token_client.transfer(
        &contract_address,
        &contract.seller,
        &(contract.escrowed_amount as i128),
    );

    let released_amount = contract.escrowed_amount;
    contract.escrowed_amount = 0;
    contract.status = ContractStatus::Completed;
    contract.completed_at = Some(env.ledger().timestamp());

    set_contract(env, &contract);

    emit_delivery_confirmed(env, contract_id, buyer.clone(), released_amount);

    Ok(())
}

pub fn request_refund(
    env: &Env,
    contract_id: u64,
    requester: &Address,
    reason: String,
) -> Result<(), ContractError> {
    let mut contract = get_contract(env, contract_id)?;

    if contract.buyer != *requester && contract.seller != *requester {
        return Err(ContractError::ParticipantOnly);
    }

    if contract.status == ContractStatus::Completed
        || contract.status == ContractStatus::Cancelled
        || contract.status == ContractStatus::RefundProcessed
    {
        return Err(ContractError::OperationNotAllowed);
    }

    if contract.status == ContractStatus::RefundRequested {
        return Err(ContractError::RefundAlreadyRequested);
    }

    let current_time = env.ledger().timestamp();
    if current_time > contract.refund_deadline {
        return Err(ContractError::RefundDeadlinePassed);
    }

    contract.status = ContractStatus::RefundRequested;
    contract.refund_requested_at = Some(current_time);
    contract.refund_reason = Some(reason.clone());
    contract.refund_requester = Some(requester.clone());

    set_contract(env, &contract);

    emit_refund_requested(env, contract_id, requester.clone(), reason);

    Ok(())
}

pub fn process_automatic_refund(env: &Env, contract_id: u64) -> Result<(), ContractError> {
    let mut contract = get_contract(env, contract_id)?;

    let current_time = env.ledger().timestamp();

    // Check if automatic refund conditions are met
    let can_auto_refund = match contract.status {
        ContractStatus::Funded => {
            // Auto-refund if delivery deadline passed and not delivered
            current_time > contract.delivery_deadline
        }
        ContractStatus::RefundRequested => {
            // Auto-refund if buyer requested and conditions met
            contract.refund_requester.as_ref() == Some(&contract.buyer)
        }
        _ => false,
    };

    if !can_auto_refund {
        return Err(ContractError::RefundConditionsNotMet);
    }

    if contract.escrowed_amount == 0 {
        return Err(ContractError::RefundAlreadyProcessed);
    }

    // Process refund to buyer
    let contract_address = env.current_contract_address();
    let token_client = token::Client::new(env, &contract.token);

    token_client.transfer(
        &contract_address,
        &contract.buyer,
        &(contract.escrowed_amount as i128),
    );

    let refunded_amount = contract.escrowed_amount;
    contract.escrowed_amount = 0;
    contract.status = ContractStatus::RefundProcessed;
    contract.refund_processed_at = Some(current_time);

    set_contract(env, &contract);

    emit_refund_processed(
        env,
        contract_id,
        contract.buyer.clone(),
        refunded_amount,
        true,
    );

    Ok(())
}

pub fn resolve_refund_dispute(
    env: &Env,
    contract_id: u64,
    admin: &Address,
    approve_refund: bool,
) -> Result<(), ContractError> {
    if !storage::is_admin(env, admin) {
        return Err(ContractError::AdminOnly);
    }

    let mut contract = get_contract(env, contract_id)?;

    if contract.status != ContractStatus::RefundRequested {
        return Err(ContractError::RefundNotRequested);
    }

    let current_time = env.ledger().timestamp();
    let contract_address = env.current_contract_address();
    let token_client = token::Client::new(env, &contract.token);
    let amount_transferred = contract.escrowed_amount;

    if approve_refund {
        // Refund to buyer
        token_client.transfer(
            &contract_address,
            &contract.buyer,
            &(contract.escrowed_amount as i128),
        );

        contract.escrowed_amount = 0;
        contract.status = ContractStatus::RefundProcessed;
        contract.refund_processed_at = Some(current_time);
    } else {
        // Release to seller
        token_client.transfer(
            &contract_address,
            &contract.seller,
            &(contract.escrowed_amount as i128),
        );

        contract.escrowed_amount = 0;
        contract.status = ContractStatus::Completed;
        contract.completed_at = Some(current_time);
    }

    set_contract(env, &contract);

    emit_refund_dispute_resolved(
        env,
        contract_id,
        admin.clone(),
        approve_refund,
        amount_transferred,
    );

    Ok(())
}

pub fn cancel_contract(
    env: &Env,
    contract_id: u64,
    canceller: &Address,
) -> Result<(), ContractError> {
    let mut contract = get_contract(env, contract_id)?;

    if contract.buyer != *canceller && contract.seller != *canceller {
        return Err(ContractError::ParticipantOnly);
    }

    if contract.status == ContractStatus::Completed
        || contract.status == ContractStatus::Cancelled
        || contract.status == ContractStatus::RefundProcessed
    {
        return Err(ContractError::OperationNotAllowed);
    }

    // Only allow cancellation before delivery is marked
    if contract.status == ContractStatus::Delivered {
        return Err(ContractError::OperationNotAllowed);
    }

    let refund_amount = contract.escrowed_amount;

    // Refund to buyer if funded
    if refund_amount > 0 {
        let contract_address = env.current_contract_address();
        let token_client = token::Client::new(env, &contract.token);

        token_client.transfer(&contract_address, &contract.buyer, &(refund_amount as i128));
    }

    contract.status = ContractStatus::Cancelled;
    contract.cancelled_at = Some(env.ledger().timestamp());
    contract.escrowed_amount = 0;

    set_contract(env, &contract);

    emit_contract_cancelled(env, contract_id, canceller.clone(), refund_amount);

    Ok(())
}

pub fn get_contract(env: &Env, contract_id: u64) -> Result<RefundContract, ContractError> {
    match crate::refund_storage::get_contract(env, contract_id) {
        Some(contract) => Ok(contract),
        None => Err(ContractError::ContractNotFound),
    }
}

pub fn get_user_contracts(
    env: &Env,
    user: &Address,
    offset: u32,
    limit: u32,
) -> Result<Vec<u64>, ContractError> {
    let all_contracts = crate::refund_storage::get_user_contracts(env, user);
    let mut result = Vec::new(env);

    let start = offset as usize;
    let end = (offset + limit) as usize;
    let contracts_len = all_contracts.len() as usize;

    for i in start..end.min(contracts_len) {
        if let Some(contract_id) = all_contracts.get(i as u32) {
            result.push_back(contract_id);
        }
    }

    Ok(result)
}
