use crate::error::ContractError;
use crate::events::*;
use crate::milestone_storage;
use crate::milestone_storage::*;
use crate::storage;
use soroban_sdk::{token, Address, Env, String, Vec};

// Validate milestone data
fn validate_milestones(milestones: &Vec<MilestoneData>, total_amount: u128) -> Result<(), ContractError> {
    if milestones.is_empty() {
        return Err(ContractError::InvalidMilestoneData);
    }

    let mut sum = 0u128;
    for milestone in milestones.iter() {
        if milestone.amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        sum = sum.checked_add(milestone.amount)
            .ok_or(ContractError::InvalidAmount)?;
    }

    if sum != total_amount {
        return Err(ContractError::TotalAmountMismatch);
    }

    Ok(())
}

pub fn create_contract(
    env: &Env,
    buyer: &Address,
    seller: &Address,
    token: &Address,
    total_amount: u128,
    milestones: Vec<MilestoneData>,
) -> Result<u64, ContractError> {
    if total_amount == 0 {
        return Err(ContractError::InvalidAmount);
    }

    if buyer == seller {
        return Err(ContractError::InvalidInput);
    }

    validate_milestones(&milestones, total_amount)?;

    let contract_id = milestone_storage::get_next_contract_id(env);
    let timestamp = env.ledger().timestamp();

    let contract = Contract {
        id: contract_id,
        buyer: buyer.clone(),
        seller: seller.clone(),
        token: token.clone(),
        total_amount,
        escrowed_amount: 0,
        released_amount: 0,
        status: ContractStatus::Created,
        created_at: timestamp,
        funded_at: None,
        completed_at: None,
        cancelled_at: None,
        milestone_count: milestones.len() as u32,
    };

    set_contract(env, &contract);
    add_user_contract(env, buyer, contract_id);
    add_user_contract(env, seller, contract_id);

    // Create milestones
    let mut milestone_ids = Vec::new(env);
    for (i, milestone_data) in milestones.iter().enumerate() {
        let milestone = Milestone {
            id: i as u32,
            description: milestone_data.description.clone(),
            amount: milestone_data.amount,
            release_criteria: milestone_data.release_criteria.clone(),
            status: MilestoneStatus::Pending,
            completed_at: None,
            approved_at: None,
            dispute_reason: None,
            disputed_at: None,
        };
        
        set_milestone(env, contract_id, &milestone);
        milestone_ids.push_back(i as u32);
    }
    
    set_contract_milestone_ids(env, contract_id, &milestone_ids);

    emit_contract_created(
        env,
        contract_id,
        buyer.clone(),
        seller.clone(),
        token.clone(),
        total_amount,
        milestones.len() as u32,
    );

    Ok(contract_id)
}

pub fn fund_contract(
    env: &Env,
    contract_id: u64,
    buyer: &Address,
) -> Result<(), ContractError> {
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
    
    token_client.transfer(buyer, &contract_address, &(contract.total_amount as i128));

    contract.escrowed_amount = contract.total_amount;
    contract.status = ContractStatus::Funded;
    contract.funded_at = Some(env.ledger().timestamp());

    set_contract(env, &contract);

    emit_contract_funded(env, contract_id, buyer.clone(), contract.total_amount);

    Ok(())
}

pub fn complete_milestone(
    env: &Env,
    contract_id: u64,
    milestone_id: u32,
    completor: &Address,
) -> Result<(), ContractError> {
    let contract = get_contract(env, contract_id)?;

    if contract.status != ContractStatus::Funded {
        return Err(ContractError::ContractNotFunded);
    }

    if contract.seller != *completor {
        return Err(ContractError::SellerOnly);
    }

    let mut milestone = get_milestone(env, contract_id, milestone_id)?;

    if milestone.status != MilestoneStatus::Pending {
        return Err(ContractError::MilestoneAlreadyCompleted);
    }

    milestone.status = MilestoneStatus::Completed;
    milestone.completed_at = Some(env.ledger().timestamp());

    set_milestone(env, contract_id, &milestone);

    emit_milestone_completed(env, contract_id, milestone_id, completor.clone());

    Ok(())
}

pub fn approve_milestone(
    env: &Env,
    contract_id: u64,
    milestone_id: u32,
    buyer: &Address,
) -> Result<(), ContractError> {
    let mut contract = get_contract(env, contract_id)?;

    if contract.buyer != *buyer {
        return Err(ContractError::BuyerOnly);
    }

    if contract.status != ContractStatus::Funded {
        return Err(ContractError::ContractNotFunded);
    }

    let mut milestone = get_milestone(env, contract_id, milestone_id)?;

    if milestone.status != MilestoneStatus::Completed {
        return Err(ContractError::MilestoneNotCompleted);
    }

    // Release funds to seller
    let contract_address = env.current_contract_address();
    let token_client = token::Client::new(env, &contract.token);
    
    token_client.transfer(&contract_address, &contract.seller, &(milestone.amount as i128));

    milestone.status = MilestoneStatus::Approved;
    milestone.approved_at = Some(env.ledger().timestamp());
    
    contract.released_amount = contract.released_amount.checked_add(milestone.amount)
        .ok_or(ContractError::InvalidAmount)?;
    contract.escrowed_amount = contract.escrowed_amount.checked_sub(milestone.amount)
        .ok_or(ContractError::InvalidAmount)?;

    set_milestone(env, contract_id, &milestone);

    // Check if all milestones are completed
    let milestone_ids = get_contract_milestone_ids(env, contract_id);
    let mut all_approved = true;
    
    for mid in milestone_ids.iter() {
        if let Ok(m) = get_milestone(env, contract_id, mid) {
            if m.status != MilestoneStatus::Approved {
                all_approved = false;
                break;
            }
        }
    }

    if all_approved {
        contract.status = ContractStatus::Completed;
        contract.completed_at = Some(env.ledger().timestamp());
        
        emit_contract_completed(
            env,
            contract_id,
            contract.buyer.clone(),
            contract.seller.clone(),
            contract.released_amount,
        );
    }

    set_contract(env, &contract);

    emit_milestone_approved(env, contract_id, milestone_id, buyer.clone(), milestone.amount);

    Ok(())
}

pub fn dispute_milestone(
    env: &Env,
    contract_id: u64,
    milestone_id: u32,
    disputer: &Address,
    reason: String,
) -> Result<(), ContractError> {
    let contract = get_contract(env, contract_id)?;

    if contract.buyer != *disputer && contract.seller != *disputer {
        return Err(ContractError::ParticipantOnly);
    }

    let mut milestone = get_milestone(env, contract_id, milestone_id)?;

    if milestone.status != MilestoneStatus::Completed {
        return Err(ContractError::MilestoneNotCompleted);
    }

    milestone.status = MilestoneStatus::Disputed;
    milestone.dispute_reason = Some(reason.clone());
    milestone.disputed_at = Some(env.ledger().timestamp());

    set_milestone(env, contract_id, &milestone);

    emit_milestone_disputed(env, contract_id, milestone_id, disputer.clone(), reason);

    Ok(())
}

pub fn resolve_dispute(
    env: &Env,
    contract_id: u64,
    milestone_id: u32,
    admin: &Address,
    approve: bool,
) -> Result<(), ContractError> {
    if !storage::is_admin(env, admin) {
        return Err(ContractError::AdminOnly);
    }

    let mut contract = get_contract(env, contract_id)?;
    let mut milestone = get_milestone(env, contract_id, milestone_id)?;

    if milestone.status != MilestoneStatus::Disputed {
        return Err(ContractError::MilestoneNotDisputed);
    }

    let mut amount_released = 0u128;

    if approve {
        // Release funds to seller
        let contract_address = env.current_contract_address();
        let token_client = token::Client::new(env, &contract.token);
        
        token_client.transfer(&contract_address, &contract.seller, &(milestone.amount as i128));

        milestone.status = MilestoneStatus::Approved;
        milestone.approved_at = Some(env.ledger().timestamp());
        
        contract.released_amount = contract.released_amount.checked_add(milestone.amount)
            .ok_or(ContractError::InvalidAmount)?;
        contract.escrowed_amount = contract.escrowed_amount.checked_sub(milestone.amount)
            .ok_or(ContractError::InvalidAmount)?;
        
        amount_released = milestone.amount;
    } else {
        milestone.status = MilestoneStatus::Resolved;
    }

    set_milestone(env, contract_id, &milestone);

    // Check if all milestones are completed when approved
    if approve {
        let milestone_ids = get_contract_milestone_ids(env, contract_id);
        let mut all_approved = true;
        
        for mid in milestone_ids.iter() {
            if let Ok(m) = get_milestone(env, contract_id, mid) {
                if m.status != MilestoneStatus::Approved {
                    all_approved = false;
                    break;
                }
            }
        }

        if all_approved {
            contract.status = ContractStatus::Completed;
            contract.completed_at = Some(env.ledger().timestamp());
            
            emit_contract_completed(
                env,
                contract_id,
                contract.buyer.clone(),
                contract.seller.clone(),
                contract.released_amount,
            );
        }
    }

    set_contract(env, &contract);

    emit_dispute_resolved(env, contract_id, milestone_id, admin.clone(), approve, amount_released);

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

    if contract.status == ContractStatus::Completed || contract.status == ContractStatus::Cancelled {
        return Err(ContractError::OperationNotAllowed);
    }

    let refund_amount = contract.escrowed_amount;

    // Refund remaining escrowed funds to buyer
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

pub fn get_contract(env: &Env, contract_id: u64) -> Result<Contract, ContractError> {
    match milestone_storage::get_contract(env, contract_id) {
        Some(contract) => Ok(contract),
        None => Err(ContractError::ContractNotFound),
    }
}

pub fn get_milestone(
    env: &Env,
    contract_id: u64,
    milestone_id: u32,
) -> Result<Milestone, ContractError> {
    match milestone_storage::get_milestone(env, contract_id, milestone_id) {
        Some(milestone) => Ok(milestone),
        None => Err(ContractError::MilestoneNotFound),
    }
}

pub fn get_contract_milestones(
    env: &Env,
    contract_id: u64,
) -> Result<Vec<Milestone>, ContractError> {
    let milestone_ids = get_contract_milestone_ids(env, contract_id);
    let mut milestones = Vec::new(env);

    for milestone_id in milestone_ids.iter() {
        let milestone = get_milestone(env, contract_id, milestone_id)?;
        milestones.push_back(milestone);
    }

    Ok(milestones)
}

pub fn get_user_contracts(
    env: &Env,
    user: &Address,
    offset: u32,
    limit: u32,
) -> Result<Vec<u64>, ContractError> {
    let all_contracts = milestone_storage::get_user_contracts(env, user);
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