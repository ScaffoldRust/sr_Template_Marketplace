#![cfg(test)]
extern crate std;

use crate::{EscrowArbitrationContract, EscrowArbitrationContractClient};
use crate::escrow_storage::EscrowStatus;
use crate::error::ContractError;
use soroban_sdk::{
    testutils::Address as _,
    token, Address, Env, String,
};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

const ESCROW_AMOUNT: u128 = 1000;
const TEST_DESCRIPTION: &str = "Test escrow transaction";

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

fn create_escrow_contract<'a>(env: &Env) -> EscrowArbitrationContractClient<'a> {
    let contract_address = env.register_contract(None, EscrowArbitrationContract);
    EscrowArbitrationContractClient::new(env, &contract_address)
}

struct EscrowTest<'a> {
    env: Env,
    admin: Address,
    buyer: Address,
    seller: Address,
    arbitrator: Address,
    token: TokenClient<'a>,
    contract: EscrowArbitrationContractClient<'a>,
    description: String,
}

impl<'a> EscrowTest<'a> {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let arbitrator = Address::generate(&env);
        let token_admin = Address::generate(&env);
        
        let (token, token_admin_client) = create_token_contract(&env, &token_admin);
        token_admin_client.mint(&buyer, &(ESCROW_AMOUNT as i128));
        
        let contract = create_escrow_contract(&env);
        let _ = contract.initialize(&admin);
        
        let description = String::from_str(&env, TEST_DESCRIPTION);
        
        EscrowTest {
            env,
            admin,
            buyer,
            seller,
            arbitrator,
            token,
            contract,
            description,
        }
    }
    
    fn create_escrow(&self) -> u64 {
        self.contract.create_escrow(
            &self.buyer,
            &self.seller,
            &self.arbitrator,
            &self.token.address,
            &ESCROW_AMOUNT,
            &self.description,
        )
    }
    
    fn deposit_funds(&self, escrow_id: u64) {
        self.contract.deposit(&escrow_id, &self.buyer);
    }
    
    fn setup_with_funded_escrow() -> (Self, u64) {
        let test = Self::setup();
        let escrow_id = test.create_escrow();
        test.deposit_funds(escrow_id);
        (test, escrow_id)
    }
}

// Initialization tests
#[test]
fn test_initialize_success() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract = create_escrow_contract(&env);
    let _result = contract.initialize(&admin);
    // Test passes if no panic occurs
}

#[test]
fn test_initialize_duplicate() {
    let test = EscrowTest::setup();
    let result = test.contract.try_initialize(&test.admin);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::AlreadyInitialized));
}

// Escrow creation tests
#[test]
fn test_create_escrow_success() {
    let test = EscrowTest::setup();
    let escrow_id = test.create_escrow();
    
    let escrow = test.contract.get_escrow(&escrow_id);
    assert_eq!(escrow.buyer, test.buyer);
    assert_eq!(escrow.seller, test.seller);
    assert_eq!(escrow.arbitrator, test.arbitrator);
    assert_eq!(escrow.amount, ESCROW_AMOUNT);
    assert_eq!(escrow.status, EscrowStatus::Created);
    assert_eq!(escrow.description, test.description);
}

#[test]
fn test_create_escrow_zero_amount() {
    let test = EscrowTest::setup();
    let result = test.contract.try_create_escrow(
        &test.buyer,
        &test.seller,
        &test.arbitrator,
        &test.token.address,
        &0,
        &test.description,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::InvalidAmount));
}

#[test]
fn test_create_escrow_same_buyer_seller() {
    let test = EscrowTest::setup();
    let result = test.contract.try_create_escrow(
        &test.buyer,
        &test.buyer,
        &test.arbitrator,
        &test.token.address,
        &ESCROW_AMOUNT,
        &test.description,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::InvalidInput));
}

#[test]
fn test_create_escrow_same_buyer_arbitrator() {
    let test = EscrowTest::setup();
    let result = test.contract.try_create_escrow(
        &test.buyer,
        &test.seller,
        &test.buyer,
        &test.token.address,
        &ESCROW_AMOUNT,
        &test.description,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::InvalidInput));
}

#[test]
fn test_create_escrow_same_seller_arbitrator() {
    let test = EscrowTest::setup();
    let result = test.contract.try_create_escrow(
        &test.buyer,
        &test.seller,
        &test.seller,
        &test.token.address,
        &ESCROW_AMOUNT,
        &test.description,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::InvalidInput));
}

// Deposit tests
#[test]
fn test_deposit_success() {
    let test = EscrowTest::setup();
    let escrow_id = test.create_escrow();
    
    let buyer_balance_before = test.token.balance(&test.buyer);
    let contract_balance_before = test.token.balance(&test.contract.address);
    
    test.deposit_funds(escrow_id);
    
    assert_eq!(
        test.token.balance(&test.buyer),
        buyer_balance_before - ESCROW_AMOUNT as i128
    );
    assert_eq!(
        test.token.balance(&test.contract.address),
        contract_balance_before + ESCROW_AMOUNT as i128
    );
    
    let escrow = test.contract.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Funded);
    assert!(escrow.funded_at.is_some());
}

#[test]
fn test_deposit_non_buyer() {
    let test = EscrowTest::setup();
    let escrow_id = test.create_escrow();
    let result = test.contract.try_deposit(&escrow_id, &test.seller);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::BuyerOnly));
}

#[test]
fn test_deposit_already_funded() {
    let test = EscrowTest::setup();
    let escrow_id = test.create_escrow();
    test.deposit_funds(escrow_id);
    
    let result = test.contract.try_deposit(&escrow_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::EscrowAlreadyFunded));
}

#[test]
fn test_deposit_nonexistent_escrow() {
    let test = EscrowTest::setup();
    let result = test.contract.try_deposit(&999, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::EscrowNotFound));
}

// Release funds tests
#[test]
fn test_release_funds_success() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    
    let seller_balance_before = test.token.balance(&test.seller);
    let contract_balance_before = test.token.balance(&test.contract.address);
    
    test.contract.release_funds(&escrow_id, &test.buyer);
    
    assert_eq!(
        test.token.balance(&test.seller),
        seller_balance_before + ESCROW_AMOUNT as i128
    );
    assert_eq!(
        test.token.balance(&test.contract.address),
        contract_balance_before - ESCROW_AMOUNT as i128
    );
    
    let escrow = test.contract.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Completed);
    assert!(escrow.completed_at.is_some());
}

#[test]
fn test_release_funds_non_buyer() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    let result = test.contract.try_release_funds(&escrow_id, &test.seller);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::BuyerOnly));
}

#[test]
fn test_release_funds_not_funded() {
    let test = EscrowTest::setup();
    let escrow_id = test.create_escrow();
    let result = test.contract.try_release_funds(&escrow_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::EscrowNotFunded));
}

// Dispute tests
#[test]
fn test_raise_dispute_by_buyer() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    
    let dispute_reason = String::from_str(&test.env, "Product not as described");
    test.contract.raise_dispute(&escrow_id, &test.buyer, &dispute_reason);
    
    let escrow = test.contract.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Disputed);
    assert!(escrow.disputed_at.is_some());
    assert_eq!(escrow.dispute_reason, Some(dispute_reason));
}

#[test]
fn test_raise_dispute_by_seller() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    
    let dispute_reason = String::from_str(&test.env, "Buyer not cooperating");
    test.contract.raise_dispute(&escrow_id, &test.seller, &dispute_reason);
    
    let escrow = test.contract.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Disputed);
    assert_eq!(escrow.dispute_reason, Some(dispute_reason));
}

#[test]
fn test_raise_dispute_non_participant() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    let outsider = Address::generate(&test.env);
    let dispute_reason = String::from_str(&test.env, "Unauthorized dispute");
    
    let result = test.contract.try_raise_dispute(&escrow_id, &outsider, &dispute_reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ParticipantOnly));
}

#[test]
fn test_raise_dispute_not_funded() {
    let test = EscrowTest::setup();
    let escrow_id = test.create_escrow();
    let dispute_reason = String::from_str(&test.env, "Invalid dispute");
    
    let result = test.contract.try_raise_dispute(&escrow_id, &test.buyer, &dispute_reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::EscrowNotFunded));
}

// Arbitration tests
#[test]
fn test_arbitrate_release_to_seller() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    
    let dispute_reason = String::from_str(&test.env, "Dispute reason");
    test.contract.raise_dispute(&escrow_id, &test.buyer, &dispute_reason);
    
    let seller_balance_before = test.token.balance(&test.seller);
    let contract_balance_before = test.token.balance(&test.contract.address);
    
    test.contract.arbitrate(&escrow_id, &test.arbitrator, &true);
    
    assert_eq!(
        test.token.balance(&test.seller),
        seller_balance_before + ESCROW_AMOUNT as i128
    );
    assert_eq!(
        test.token.balance(&test.contract.address),
        contract_balance_before - ESCROW_AMOUNT as i128
    );
    
    let escrow = test.contract.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Completed);
    assert!(escrow.completed_at.is_some());
}

#[test]
fn test_arbitrate_refund_to_buyer() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    
    let dispute_reason = String::from_str(&test.env, "Dispute reason");
    test.contract.raise_dispute(&escrow_id, &test.buyer, &dispute_reason);
    
    let buyer_balance_before = test.token.balance(&test.buyer);
    let contract_balance_before = test.token.balance(&test.contract.address);
    
    test.contract.arbitrate(&escrow_id, &test.arbitrator, &false);
    
    assert_eq!(
        test.token.balance(&test.buyer),
        buyer_balance_before + ESCROW_AMOUNT as i128
    );
    assert_eq!(
        test.token.balance(&test.contract.address),
        contract_balance_before - ESCROW_AMOUNT as i128
    );
    
    let escrow = test.contract.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Completed);
}

#[test]
fn test_arbitrate_non_arbitrator() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    
    let dispute_reason = String::from_str(&test.env, "Dispute reason");
    test.contract.raise_dispute(&escrow_id, &test.buyer, &dispute_reason);
    
    let result = test.contract.try_arbitrate(&escrow_id, &test.buyer, &true);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ArbitratorOnly));
}

#[test]
fn test_arbitrate_not_disputed() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    
    let result = test.contract.try_arbitrate(&escrow_id, &test.arbitrator, &true);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::EscrowNotDisputed));
}

// Refund tests
#[test]
fn test_refund_by_buyer() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    
    let buyer_balance_before = test.token.balance(&test.buyer);
    let contract_balance_before = test.token.balance(&test.contract.address);
    
    test.contract.refund(&escrow_id, &test.buyer);
    
    assert_eq!(
        test.token.balance(&test.buyer),
        buyer_balance_before + ESCROW_AMOUNT as i128
    );
    assert_eq!(
        test.token.balance(&test.contract.address),
        contract_balance_before - ESCROW_AMOUNT as i128
    );
    
    let escrow = test.contract.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Cancelled);
    assert!(escrow.completed_at.is_some());
}

#[test]
fn test_refund_by_seller() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    
    let buyer_balance_before = test.token.balance(&test.buyer);
    
    test.contract.refund(&escrow_id, &test.seller);
    
    assert_eq!(
        test.token.balance(&test.buyer),
        buyer_balance_before + ESCROW_AMOUNT as i128
    );
    
    let escrow = test.contract.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Cancelled);
}

#[test]
fn test_refund_non_participant() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    let outsider = Address::generate(&test.env);
    
    let result = test.contract.try_refund(&escrow_id, &outsider);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ParticipantOnly));
}

#[test]
fn test_refund_not_funded() {
    let test = EscrowTest::setup();
    let escrow_id = test.create_escrow();
    
    let result = test.contract.try_refund(&escrow_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::OperationNotAllowed));
}

#[test]
fn test_refund_disputed_escrow() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    
    let dispute_reason = String::from_str(&test.env, "Dispute reason");
    test.contract.raise_dispute(&escrow_id, &test.buyer, &dispute_reason);
    
    let result = test.contract.try_refund(&escrow_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::OperationNotAllowed));
}

// Edge case tests
#[test]
fn test_duplicate_dispute_attempt() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    
    let dispute_reason1 = String::from_str(&test.env, "First dispute");
    test.contract.raise_dispute(&escrow_id, &test.buyer, &dispute_reason1);
    
    let dispute_reason2 = String::from_str(&test.env, "Second dispute");
    let result = test.contract.try_raise_dispute(&escrow_id, &test.seller, &dispute_reason2);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::EscrowNotFunded));
}

#[test]
fn test_multiple_arbitration_attempts() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    
    let dispute_reason = String::from_str(&test.env, "Dispute reason");
    test.contract.raise_dispute(&escrow_id, &test.buyer, &dispute_reason);
    
    test.contract.arbitrate(&escrow_id, &test.arbitrator, &true);
    
    // Try to arbitrate again
    let result = test.contract.try_arbitrate(&escrow_id, &test.arbitrator, &false);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::EscrowNotDisputed));
}

#[test]
fn test_operations_on_completed_escrow() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    
    // Complete the escrow
    test.contract.release_funds(&escrow_id, &test.buyer);
    
    // Try various operations on completed escrow
    let result = test.contract.try_release_funds(&escrow_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::EscrowNotFunded));
    
    let dispute_reason = String::from_str(&test.env, "Invalid dispute");
    let result = test.contract.try_raise_dispute(&escrow_id, &test.buyer, &dispute_reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::EscrowNotFunded));
    
    let result = test.contract.try_refund(&escrow_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::OperationNotAllowed));
}

#[test]
fn test_get_user_escrows_pagination() {
    let test = EscrowTest::setup();
    
    // Create multiple escrows
    let mut escrow_ids = std::vec![];
    for _ in 0..5 {
        let escrow_id = test.create_escrow();
        escrow_ids.push(escrow_id);
    }
    
    // Test pagination
    let first_page = test.contract.get_user_escrows(&test.buyer, &0, &3);
    assert_eq!(first_page.len(), 3);
    
    let second_page = test.contract.get_user_escrows(&test.buyer, &3, &3);
    assert_eq!(second_page.len(), 2);
    
    // Test seller's escrows
    let seller_escrows = test.contract.get_user_escrows(&test.seller, &0, &10);
    assert_eq!(seller_escrows.len(), 5);
    
    // Test arbitrator's escrows
    let arbitrator_escrows = test.contract.get_user_escrows(&test.arbitrator, &0, &10);
    assert_eq!(arbitrator_escrows.len(), 5);
}

#[test]
fn test_unauthorized_actions_protection() {
    let (test, escrow_id) = EscrowTest::setup_with_funded_escrow();
    let unauthorized = Address::generate(&test.env);
    
    // Test various unauthorized access attempts
    let result = test.contract.try_release_funds(&escrow_id, &unauthorized);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::BuyerOnly));
    
    let dispute_reason = String::from_str(&test.env, "Unauthorized dispute");
    let result = test.contract.try_raise_dispute(&escrow_id, &unauthorized, &dispute_reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ParticipantOnly));
    
    let result = test.contract.try_refund(&escrow_id, &unauthorized);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ParticipantOnly));
}