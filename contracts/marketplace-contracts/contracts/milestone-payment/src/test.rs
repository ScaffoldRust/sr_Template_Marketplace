#![cfg(test)]
extern crate std;

use crate::{MilestonePaymentContract, MilestonePaymentContractClient};
use crate::milestone_storage::{MilestoneData, MilestoneStatus, ContractStatus};
use crate::error::ContractError;
use soroban_sdk::{
    vec,
    testutils::Address as _,
    token, Address, Env, String,
};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

const MILESTONE_AMOUNT_1: u128 = 500;
const MILESTONE_AMOUNT_2: u128 = 300;
const MILESTONE_AMOUNT_3: u128 = 200;
const TOTAL_AMOUNT: u128 = MILESTONE_AMOUNT_1 + MILESTONE_AMOUNT_2 + MILESTONE_AMOUNT_3;

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(env, &sac.address()),
        token::StellarAssetClient::new(env, &sac.address()),
    )
}

fn create_milestone_contract<'a>(env: &Env) -> MilestonePaymentContractClient<'a> {
    let contract_address = env.register_contract(None, MilestonePaymentContract);
    MilestonePaymentContractClient::new(env, &contract_address)
}

struct MilestoneTest<'a> {
    env: Env,
    admin: Address,
    buyer: Address,
    seller: Address,
    token: TokenClient<'a>,
    contract: MilestonePaymentContractClient<'a>,
    milestones: soroban_sdk::Vec<MilestoneData>,
}

impl<'a> MilestoneTest<'a> {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let token_admin = Address::generate(&env);
        
        let (token, token_admin_client) = create_token_contract(&env, &token_admin);
        token_admin_client.mint(&buyer, &(TOTAL_AMOUNT as i128));
        
        let contract = create_milestone_contract(&env);
        let _ = contract.initialize(&admin);
        
        let mut milestones = vec![&env];
        milestones.push_back(MilestoneData {
            description: String::from_str(&env, "First milestone"),
            amount: MILESTONE_AMOUNT_1,
            release_criteria: String::from_str(&env, "Complete development phase 1"),
        });
        milestones.push_back(MilestoneData {
            description: String::from_str(&env, "Second milestone"),
            amount: MILESTONE_AMOUNT_2,
            release_criteria: String::from_str(&env, "Complete testing phase"),
        });
        milestones.push_back(MilestoneData {
            description: String::from_str(&env, "Final milestone"),
            amount: MILESTONE_AMOUNT_3,
            release_criteria: String::from_str(&env, "Complete deployment"),
        });
        
        MilestoneTest {
            env,
            admin,
            buyer,
            seller,
            token,
            contract,
            milestones,
        }
    }
    
    fn create_contract(&self) -> u64 {
        self.contract.create_contract(
            &self.buyer,
            &self.seller,
            &self.token.address,
            &TOTAL_AMOUNT,
            &self.milestones,
        )
    }
    
    fn fund_contract(&self, contract_id: u64) {
        self.contract.fund_contract(&contract_id, &self.buyer);
    }
    
    fn setup_with_funded_contract() -> (Self, u64) {
        let test = Self::setup();
        let contract_id = test.create_contract();
        test.fund_contract(contract_id);
        (test, contract_id)
    }
}

// Initialization tests
#[test]
fn test_initialize_success() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract = create_milestone_contract(&env);
    let _result = contract.initialize(&admin);
    // The test passes if no panic occurs
}

#[test]
fn test_initialize_duplicate() {
    let test = MilestoneTest::setup();
    // Try to initialize again - this should fail
    let result = test.contract.try_initialize(&test.admin);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::AlreadyInitialized));
}

// Contract creation tests
#[test]
fn test_create_contract_success() {
    let test = MilestoneTest::setup();
    let contract_id = test.create_contract();
    
    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.buyer, test.buyer);
    assert_eq!(contract_data.seller, test.seller);
    assert_eq!(contract_data.total_amount, TOTAL_AMOUNT);
    assert_eq!(contract_data.status, ContractStatus::Created);
    assert_eq!(contract_data.milestone_count, 3);
}

#[test]
fn test_create_contract_zero_amount() {
    let test = MilestoneTest::setup();
    let result = test.contract.try_create_contract(
        &test.buyer,
        &test.seller,
        &test.token.address,
        &0,
        &test.milestones,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::InvalidAmount));
}

#[test]
fn test_create_contract_same_buyer_seller() {
    let test = MilestoneTest::setup();
    let result = test.contract.try_create_contract(
        &test.buyer,
        &test.buyer,
        &test.token.address,
        &TOTAL_AMOUNT,
        &test.milestones,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::InvalidInput));
}

#[test]
fn test_create_contract_milestone_amount_mismatch() {
    let test = MilestoneTest::setup();
    let wrong_total = TOTAL_AMOUNT + 100;
    let result = test.contract.try_create_contract(
        &test.buyer,
        &test.seller,
        &test.token.address,
        &wrong_total,
        &test.milestones,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::TotalAmountMismatch));
}

#[test]
fn test_create_contract_empty_milestones() {
    let test = MilestoneTest::setup();
    let empty_milestones = vec![&test.env];
    let result = test.contract.try_create_contract(
        &test.buyer,
        &test.seller,
        &test.token.address,
        &TOTAL_AMOUNT,
        &empty_milestones,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::InvalidMilestoneData));
}

#[test]
fn test_create_contract_zero_milestone_amount() {
    let test = MilestoneTest::setup();
    let mut invalid_milestones = vec![&test.env];
    invalid_milestones.push_back(MilestoneData {
        description: String::from_str(&test.env, "Invalid milestone"),
        amount: 0,
        release_criteria: String::from_str(&test.env, "Invalid criteria"),
    });
    
    let result = test.contract.try_create_contract(
        &test.buyer,
        &test.seller,
        &test.token.address,
        &TOTAL_AMOUNT,
        &invalid_milestones,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::InvalidAmount));
}

// Fund contract tests
#[test]
fn test_fund_contract_success() {
    let test = MilestoneTest::setup();
    let contract_id = test.create_contract();
    
    let buyer_balance_before = test.token.balance(&test.buyer);
    let contract_balance_before = test.token.balance(&test.contract.address);
    
    test.fund_contract(contract_id);
    
    assert_eq!(
        test.token.balance(&test.buyer),
        buyer_balance_before - TOTAL_AMOUNT as i128
    );
    assert_eq!(
        test.token.balance(&test.contract.address),
        contract_balance_before + TOTAL_AMOUNT as i128
    );
    
    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.status, ContractStatus::Funded);
    assert_eq!(contract_data.escrowed_amount, TOTAL_AMOUNT);
}

#[test]
fn test_fund_contract_non_buyer() {
    let test = MilestoneTest::setup();
    let contract_id = test.create_contract();
    let result = test.contract.try_fund_contract(&contract_id, &test.seller);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::BuyerOnly));
}

#[test]
fn test_fund_contract_already_funded() {
    let test = MilestoneTest::setup();
    let contract_id = test.create_contract();
    test.fund_contract(contract_id);
    
    let result = test.contract.try_fund_contract(&contract_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ContractAlreadyFunded));
}

#[test]
fn test_fund_nonexistent_contract() {
    let test = MilestoneTest::setup();
    let result = test.contract.try_fund_contract(&999, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ContractNotFound));
}

// Milestone completion tests
#[test]
fn test_complete_milestone_success() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    test.contract.complete_milestone(&contract_id, &0, &test.seller);
    
    let milestone = test.contract.get_milestone(&contract_id, &0);
    assert_eq!(milestone.status, MilestoneStatus::Completed);
    assert!(milestone.completed_at.is_some());
}

#[test]
fn test_complete_milestone_unfunded_contract() {
    let test = MilestoneTest::setup();
    let contract_id = test.create_contract();
    let result = test.contract.try_complete_milestone(&contract_id, &0, &test.seller);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ContractNotFunded));
}

#[test]
fn test_complete_milestone_non_seller() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    let result = test.contract.try_complete_milestone(&contract_id, &0, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::SellerOnly));
}

#[test]
fn test_complete_milestone_already_completed() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    test.contract.complete_milestone(&contract_id, &0, &test.seller);
    
    let result = test.contract.try_complete_milestone(&contract_id, &0, &test.seller);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::MilestoneAlreadyCompleted));
}

#[test]
fn test_complete_nonexistent_milestone() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    let result = test.contract.try_complete_milestone(&contract_id, &999, &test.seller);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::MilestoneNotFound));
}

// Milestone approval and fund release tests
#[test]
fn test_approve_milestone_success() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    test.contract.complete_milestone(&contract_id, &0, &test.seller);
    
    let seller_balance_before = test.token.balance(&test.seller);
    let contract_balance_before = test.token.balance(&test.contract.address);
    
    test.contract.approve_milestone(&contract_id, &0, &test.buyer);
    
    assert_eq!(
        test.token.balance(&test.seller),
        seller_balance_before + MILESTONE_AMOUNT_1 as i128
    );
    assert_eq!(
        test.token.balance(&test.contract.address),
        contract_balance_before - MILESTONE_AMOUNT_1 as i128
    );
    
    let milestone = test.contract.get_milestone(&contract_id, &0);
    assert_eq!(milestone.status, MilestoneStatus::Approved);
    
    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.released_amount, MILESTONE_AMOUNT_1);
    assert_eq!(contract_data.escrowed_amount, TOTAL_AMOUNT - MILESTONE_AMOUNT_1);
}

#[test]
fn test_approve_milestone_non_buyer() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    test.contract.complete_milestone(&contract_id, &0, &test.seller);
    
    let result = test.contract.try_approve_milestone(&contract_id, &0, &test.seller);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::BuyerOnly));
}

#[test]
fn test_approve_milestone_not_completed() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    let result = test.contract.try_approve_milestone(&contract_id, &0, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::MilestoneNotCompleted));
}

#[test]
fn test_complete_all_milestones_contract_completion() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    // Complete and approve all milestones
    for i in 0..3u32 {
        test.contract.complete_milestone(&contract_id, &i, &test.seller);
        test.contract.approve_milestone(&contract_id, &i, &test.buyer);
    }
    
    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.status, ContractStatus::Completed);
    assert_eq!(contract_data.released_amount, TOTAL_AMOUNT);
    assert_eq!(contract_data.escrowed_amount, 0);
    assert!(contract_data.completed_at.is_some());
}

// Dispute tests
#[test]
fn test_dispute_milestone_success() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    test.contract.complete_milestone(&contract_id, &0, &test.seller);
    
    let dispute_reason = String::from_str(&test.env, "Work not satisfactory");
    test.contract.dispute_milestone(&contract_id, &0, &test.buyer, &dispute_reason);
    
    let milestone = test.contract.get_milestone(&contract_id, &0);
    assert_eq!(milestone.status, MilestoneStatus::Disputed);
    assert_eq!(milestone.dispute_reason, Some(dispute_reason));
    assert!(milestone.disputed_at.is_some());
}

#[test]
fn test_dispute_milestone_non_participant() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    test.contract.complete_milestone(&contract_id, &0, &test.seller);
    
    let outsider = Address::generate(&test.env);
    let dispute_reason = String::from_str(&test.env, "Unauthorized dispute");
    let result = test.contract.try_dispute_milestone(&contract_id, &0, &outsider, &dispute_reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ParticipantOnly));
}

#[test]
fn test_dispute_milestone_not_completed() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    let dispute_reason = String::from_str(&test.env, "Premature dispute");
    let result = test.contract.try_dispute_milestone(&contract_id, &0, &test.buyer, &dispute_reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::MilestoneNotCompleted));
}

// Dispute resolution tests
#[test]
fn test_resolve_dispute_approve() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    test.contract.complete_milestone(&contract_id, &0, &test.seller);
    let dispute_reason = String::from_str(&test.env, "Work not satisfactory");
    test.contract.dispute_milestone(&contract_id, &0, &test.buyer, &dispute_reason);
    
    let seller_balance_before = test.token.balance(&test.seller);
    
    test.contract.resolve_dispute(&contract_id, &0, &test.admin, &true);
    
    assert_eq!(
        test.token.balance(&test.seller),
        seller_balance_before + MILESTONE_AMOUNT_1 as i128
    );
    
    let milestone = test.contract.get_milestone(&contract_id, &0);
    assert_eq!(milestone.status, MilestoneStatus::Approved);
}

#[test]
fn test_resolve_dispute_reject() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    test.contract.complete_milestone(&contract_id, &0, &test.seller);
    let dispute_reason = String::from_str(&test.env, "Work not satisfactory");
    test.contract.dispute_milestone(&contract_id, &0, &test.buyer, &dispute_reason);
    
    let seller_balance_before = test.token.balance(&test.seller);
    
    test.contract.resolve_dispute(&contract_id, &0, &test.admin, &false);
    
    assert_eq!(test.token.balance(&test.seller), seller_balance_before);
    
    let milestone = test.contract.get_milestone(&contract_id, &0);
    assert_eq!(milestone.status, MilestoneStatus::Resolved);
}

#[test]
fn test_resolve_dispute_non_admin() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    test.contract.complete_milestone(&contract_id, &0, &test.seller);
    let dispute_reason = String::from_str(&test.env, "Work not satisfactory");
    test.contract.dispute_milestone(&contract_id, &0, &test.buyer, &dispute_reason);
    
    let result = test.contract.try_resolve_dispute(&contract_id, &0, &test.buyer, &true);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::AdminOnly));
}

#[test]
fn test_resolve_dispute_not_disputed() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    test.contract.complete_milestone(&contract_id, &0, &test.seller);
    let result = test.contract.try_resolve_dispute(&contract_id, &0, &test.admin, &true);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::MilestoneNotDisputed));
}

// Contract cancellation tests
#[test]
fn test_cancel_contract_success() {
    let test = MilestoneTest::setup();
    let contract_id = test.create_contract();
    test.fund_contract(contract_id);
    
    let buyer_balance_before = test.token.balance(&test.buyer);
    
    test.contract.cancel_contract(&contract_id, &test.buyer);
    
    assert_eq!(
        test.token.balance(&test.buyer),
        buyer_balance_before + TOTAL_AMOUNT as i128
    );
    
    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.status, ContractStatus::Cancelled);
    assert_eq!(contract_data.escrowed_amount, 0);
    assert!(contract_data.cancelled_at.is_some());
}

#[test]
fn test_cancel_contract_partial_refund() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    // Complete and approve one milestone
    test.contract.complete_milestone(&contract_id, &0, &test.seller);
    test.contract.approve_milestone(&contract_id, &0, &test.buyer);
    
    let buyer_balance_before = test.token.balance(&test.buyer);
    let expected_refund = TOTAL_AMOUNT - MILESTONE_AMOUNT_1;
    
    test.contract.cancel_contract(&contract_id, &test.buyer);
    
    assert_eq!(
        test.token.balance(&test.buyer),
        buyer_balance_before + expected_refund as i128
    );
}

#[test]
fn test_cancel_contract_non_participant() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    let outsider = Address::generate(&test.env);
    let result = test.contract.try_cancel_contract(&contract_id, &outsider);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ParticipantOnly));
}

#[test]
fn test_cancel_completed_contract() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    // Complete all milestones
    for i in 0..3u32 {
        test.contract.complete_milestone(&contract_id, &i, &test.seller);
        test.contract.approve_milestone(&contract_id, &i, &test.buyer);
    }
    
    let result = test.contract.try_cancel_contract(&contract_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::OperationNotAllowed));
}

#[test]
fn test_cancel_already_cancelled_contract() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    test.contract.cancel_contract(&contract_id, &test.buyer);
    let result = test.contract.try_cancel_contract(&contract_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::OperationNotAllowed));
}

// Edge case tests
#[test]
fn test_milestone_exceeding_total_payment() {
    let test = MilestoneTest::setup();
    let mut excessive_milestones = vec![&test.env];
    excessive_milestones.push_back(MilestoneData {
        description: String::from_str(&test.env, "Excessive milestone"),
        amount: TOTAL_AMOUNT + 1,
        release_criteria: String::from_str(&test.env, "Too much money"),
    });
    
    let result = test.contract.try_create_contract(
        &test.buyer,
        &test.seller,
        &test.token.address,
        &TOTAL_AMOUNT,
        &excessive_milestones,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::TotalAmountMismatch));
}

#[test]
fn test_get_user_contracts_pagination() {
    let test = MilestoneTest::setup();
    
    // Create multiple contracts
    let mut contract_ids = std::vec![];
    for _ in 0..5 {
        let contract_id = test.create_contract();
        contract_ids.push(contract_id);
    }
    
    // Test pagination
    let first_page = test.contract.get_user_contracts(&test.buyer, &0, &3);
    assert_eq!(first_page.len(), 3);
    
    let second_page = test.contract.get_user_contracts(&test.buyer, &3, &3);
    assert_eq!(second_page.len(), 2);
}

#[test]
fn test_get_contract_milestones() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    
    let milestones = test.contract.get_contract_milestones(&contract_id);
    assert_eq!(milestones.len(), 3);
    
    for (i, milestone) in milestones.iter().enumerate() {
        assert_eq!(milestone.id, i as u32);
        assert_eq!(milestone.status, MilestoneStatus::Pending);
    }
}

#[test]
fn test_fund_contract_insufficient_balance() {
    let test = MilestoneTest::setup();
    
    // Create buyer with insufficient balance
    let poor_buyer = Address::generate(&test.env);
    
    let contract_id = test.contract.create_contract(
        &poor_buyer,
        &test.seller,
        &test.token.address,
        &TOTAL_AMOUNT,
        &test.milestones,
    );
    
    // This should fail due to insufficient balance
    let result = test.contract.try_fund_contract(&contract_id, &poor_buyer);
    assert!(result.is_err());
    // The exact error will be a host error from the token contract, not our ContractError
}

#[test]
fn test_unauthorized_access_protection() {
    let (test, contract_id) = MilestoneTest::setup_with_funded_contract();
    let unauthorized = Address::generate(&test.env);
    
    // Test various unauthorized access attempts
    let result = test.contract.try_complete_milestone(&contract_id, &0, &unauthorized);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::SellerOnly));
    
    test.contract.complete_milestone(&contract_id, &0, &test.seller);
    
    let result = test.contract.try_approve_milestone(&contract_id, &0, &unauthorized);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::BuyerOnly));
    
    let dispute_reason = String::from_str(&test.env, "Unauthorized dispute");
    let result = test.contract.try_dispute_milestone(&contract_id, &0, &unauthorized, &dispute_reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ParticipantOnly));
}