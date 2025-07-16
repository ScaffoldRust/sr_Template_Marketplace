#![cfg(test)]
extern crate std;

use crate::error::ContractError;
use crate::refund_storage::ContractStatus;
use crate::{ConditionalRefundContract, ConditionalRefundContractClient};
use soroban_sdk::{Address, Env, String, testutils::Address as _, testutils::Ledger, token};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

const REFUND_AMOUNT: u128 = 1000;
const REFUND_DEADLINE: u64 = 1000;
const DELIVERY_DEADLINE: u64 = 800;

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

fn create_refund_contract<'a>(env: &Env) -> ConditionalRefundContractClient<'a> {
    let contract_address = env.register_contract(None, ConditionalRefundContract);
    ConditionalRefundContractClient::new(env, &contract_address)
}

struct RefundTest<'a> {
    env: Env,
    admin: Address,
    buyer: Address,
    seller: Address,
    token: TokenClient<'a>,
    contract: ConditionalRefundContractClient<'a>,
    refund_conditions: String,
}

impl<'a> RefundTest<'a> {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let token_admin = Address::generate(&env);

        let (token, token_admin_client) = create_token_contract(&env, &token_admin);
        token_admin_client.mint(&buyer, &(REFUND_AMOUNT as i128));

        let contract = create_refund_contract(&env);
        let _ = contract.initialize(&admin);

        let refund_conditions = String::from_str(&env, "Defective product or not as described");

        RefundTest {
            env,
            admin,
            buyer,
            seller,
            token,
            contract,
            refund_conditions,
        }
    }

    fn create_refund_contract(&self) -> u64 {
        self.contract.create_refund_contract(
            &self.buyer,
            &self.seller,
            &self.token.address,
            &REFUND_AMOUNT,
            &REFUND_DEADLINE,
            &DELIVERY_DEADLINE,
            &self.refund_conditions,
        )
    }

    fn fund_contract(&self, contract_id: u64) {
        self.contract.fund_contract(&contract_id, &self.buyer);
    }

    fn setup_with_funded_contract() -> (Self, u64) {
        let test = Self::setup();
        let contract_id = test.create_refund_contract();
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
    let contract = create_refund_contract(&env);
    let _result = contract.initialize(&admin);
    // The test passes if no panic occurs
}

#[test]
fn test_initialize_duplicate() {
    let test = RefundTest::setup();
    let result = test.contract.try_initialize(&test.admin);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::AlreadyInitialized));
}

// Contract creation tests
#[test]
fn test_create_refund_contract_success() {
    let test = RefundTest::setup();
    let contract_id = test.create_refund_contract();

    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.buyer, test.buyer);
    assert_eq!(contract_data.seller, test.seller);
    assert_eq!(contract_data.amount, REFUND_AMOUNT);
    assert_eq!(contract_data.status, ContractStatus::Created);
    assert_eq!(contract_data.refund_deadline, REFUND_DEADLINE);
    assert_eq!(contract_data.delivery_deadline, DELIVERY_DEADLINE);
}

#[test]
fn test_create_refund_contract_zero_amount() {
    let test = RefundTest::setup();
    let result = test.contract.try_create_refund_contract(
        &test.buyer,
        &test.seller,
        &test.token.address,
        &0,
        &REFUND_DEADLINE,
        &DELIVERY_DEADLINE,
        &test.refund_conditions,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::InvalidAmount));
}

#[test]
fn test_create_refund_contract_same_buyer_seller() {
    let test = RefundTest::setup();
    let result = test.contract.try_create_refund_contract(
        &test.buyer,
        &test.buyer,
        &test.token.address,
        &REFUND_AMOUNT,
        &REFUND_DEADLINE,
        &DELIVERY_DEADLINE,
        &test.refund_conditions,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::InvalidInput));
}

#[test]
fn test_create_refund_contract_deadline_in_past() {
    let test = RefundTest::setup();
    let current_time = test.env.ledger().timestamp();
    let past_deadline = current_time.saturating_sub(100);
    let result = test.contract.try_create_refund_contract(
        &test.buyer,
        &test.seller,
        &test.token.address,
        &REFUND_AMOUNT,
        &past_deadline,
        &DELIVERY_DEADLINE,
        &test.refund_conditions,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::DeadlineInPast));
}

// Fund contract tests
#[test]
fn test_fund_contract_success() {
    let test = RefundTest::setup();
    let contract_id = test.create_refund_contract();

    let buyer_balance_before = test.token.balance(&test.buyer);
    let contract_balance_before = test.token.balance(&test.contract.address);

    test.fund_contract(contract_id);

    assert_eq!(
        test.token.balance(&test.buyer),
        buyer_balance_before - REFUND_AMOUNT as i128
    );
    assert_eq!(
        test.token.balance(&test.contract.address),
        contract_balance_before + REFUND_AMOUNT as i128
    );

    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.status, ContractStatus::Funded);
    assert_eq!(contract_data.escrowed_amount, REFUND_AMOUNT);
}

#[test]
fn test_fund_contract_non_buyer() {
    let test = RefundTest::setup();
    let contract_id = test.create_refund_contract();
    let result = test.contract.try_fund_contract(&contract_id, &test.seller);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::BuyerOnly));
}

#[test]
fn test_fund_contract_already_funded() {
    let test = RefundTest::setup();
    let contract_id = test.create_refund_contract();
    test.fund_contract(contract_id);

    let result = test.contract.try_fund_contract(&contract_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        Ok(ContractError::ContractAlreadyFunded)
    );
}

#[test]
fn test_fund_nonexistent_contract() {
    let test = RefundTest::setup();
    let result = test.contract.try_fund_contract(&999, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ContractNotFound));
}

// Delivery tests
#[test]
fn test_mark_delivered_success() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    test.contract.mark_delivered(&contract_id, &test.seller);

    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.status, ContractStatus::Delivered);
    assert!(contract_data.delivered_at.is_some());
}

#[test]
fn test_mark_delivered_unfunded_contract() {
    let test = RefundTest::setup();
    let contract_id = test.create_refund_contract();
    let result = test.contract.try_mark_delivered(&contract_id, &test.seller);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ContractNotFunded));
}

#[test]
fn test_mark_delivered_non_seller() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();
    let result = test.contract.try_mark_delivered(&contract_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::SellerOnly));
}

#[test]
fn test_mark_delivered_after_deadline() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    // Jump past delivery deadline
    test.env.ledger().with_mut(|li| {
        li.timestamp = DELIVERY_DEADLINE + 100;
    });

    let result = test.contract.try_mark_delivered(&contract_id, &test.seller);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        Ok(ContractError::DeliveryDeadlinePassed)
    );
}

// Delivery confirmation tests
#[test]
fn test_confirm_delivery_success() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    test.contract.mark_delivered(&contract_id, &test.seller);

    let seller_balance_before = test.token.balance(&test.seller);
    let contract_balance_before = test.token.balance(&test.contract.address);

    test.contract.confirm_delivery(&contract_id, &test.buyer);

    assert_eq!(
        test.token.balance(&test.seller),
        seller_balance_before + REFUND_AMOUNT as i128
    );
    assert_eq!(
        test.token.balance(&test.contract.address),
        contract_balance_before - REFUND_AMOUNT as i128
    );

    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.status, ContractStatus::Completed);
    assert_eq!(contract_data.escrowed_amount, 0);
}

#[test]
fn test_confirm_delivery_non_buyer() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();
    test.contract.mark_delivered(&contract_id, &test.seller);

    let result = test
        .contract
        .try_confirm_delivery(&contract_id, &test.seller);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::BuyerOnly));
}

#[test]
fn test_confirm_delivery_not_marked() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();
    let result = test
        .contract
        .try_confirm_delivery(&contract_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::DeliveryNotMarked));
}

// Refund request tests
#[test]
fn test_request_refund_success() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    let refund_reason = String::from_str(&test.env, "Product defective");
    test.contract
        .request_refund(&contract_id, &test.buyer, &refund_reason);

    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.status, ContractStatus::RefundRequested);
    assert_eq!(contract_data.refund_reason, Some(refund_reason));
    assert_eq!(contract_data.refund_requester, Some(test.buyer.clone()));
}

#[test]
fn test_request_refund_non_participant() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    let outsider = Address::generate(&test.env);
    let refund_reason = String::from_str(&test.env, "Unauthorized refund");
    let result = test
        .contract
        .try_request_refund(&contract_id, &outsider, &refund_reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ParticipantOnly));
}

#[test]
fn test_request_refund_completed_contract() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    test.contract.mark_delivered(&contract_id, &test.seller);
    test.contract.confirm_delivery(&contract_id, &test.buyer);

    let refund_reason = String::from_str(&test.env, "Too late");
    let result = test
        .contract
        .try_request_refund(&contract_id, &test.buyer, &refund_reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::OperationNotAllowed));
}

#[test]
fn test_request_refund_already_requested() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    let refund_reason = String::from_str(&test.env, "First request");
    test.contract
        .request_refund(&contract_id, &test.buyer, &refund_reason);

    let second_reason = String::from_str(&test.env, "Second request");
    let result = test
        .contract
        .try_request_refund(&contract_id, &test.buyer, &second_reason);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        Ok(ContractError::RefundAlreadyRequested)
    );
}

#[test]
fn test_request_refund_deadline_passed() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    // Jump past refund deadline
    test.env.ledger().with_mut(|li| {
        li.timestamp = REFUND_DEADLINE + 100;
    });

    let refund_reason = String::from_str(&test.env, "Too late");
    let result = test
        .contract
        .try_request_refund(&contract_id, &test.buyer, &refund_reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::RefundDeadlinePassed));
}

// Automatic refund tests
#[test]
fn test_process_automatic_refund_delivery_deadline_passed() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    // Jump past delivery deadline
    test.env.ledger().with_mut(|li| {
        li.timestamp = DELIVERY_DEADLINE + 100;
    });

    let buyer_balance_before = test.token.balance(&test.buyer);

    test.contract.process_automatic_refund(&contract_id);

    assert_eq!(
        test.token.balance(&test.buyer),
        buyer_balance_before + REFUND_AMOUNT as i128
    );

    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.status, ContractStatus::RefundProcessed);
    assert_eq!(contract_data.escrowed_amount, 0);
}

#[test]
fn test_process_automatic_refund_buyer_requested() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    let refund_reason = String::from_str(&test.env, "Product defective");
    test.contract
        .request_refund(&contract_id, &test.buyer, &refund_reason);

    let buyer_balance_before = test.token.balance(&test.buyer);

    test.contract.process_automatic_refund(&contract_id);

    assert_eq!(
        test.token.balance(&test.buyer),
        buyer_balance_before + REFUND_AMOUNT as i128
    );

    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.status, ContractStatus::RefundProcessed);
}

#[test]
fn test_process_automatic_refund_conditions_not_met() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    let result = test.contract.try_process_automatic_refund(&contract_id);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        Ok(ContractError::RefundConditionsNotMet)
    );
}

#[test]
fn test_process_automatic_refund_already_processed() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    // Jump past delivery deadline and process refund
    test.env.ledger().with_mut(|li| {
        li.timestamp = DELIVERY_DEADLINE + 100;
    });

    test.contract.process_automatic_refund(&contract_id);

    let result = test.contract.try_process_automatic_refund(&contract_id);
    assert!(result.is_err());
}

// Dispute resolution tests
#[test]
fn test_resolve_refund_dispute_approve() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    let refund_reason = String::from_str(&test.env, "Dispute reason");
    test.contract
        .request_refund(&contract_id, &test.buyer, &refund_reason);

    let buyer_balance_before = test.token.balance(&test.buyer);

    test.contract
        .resolve_refund_dispute(&contract_id, &test.admin, &true);

    assert_eq!(
        test.token.balance(&test.buyer),
        buyer_balance_before + REFUND_AMOUNT as i128
    );

    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.status, ContractStatus::RefundProcessed);
}

#[test]
fn test_resolve_refund_dispute_reject() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    let refund_reason = String::from_str(&test.env, "Dispute reason");
    test.contract
        .request_refund(&contract_id, &test.buyer, &refund_reason);

    let seller_balance_before = test.token.balance(&test.seller);

    test.contract
        .resolve_refund_dispute(&contract_id, &test.admin, &false);

    assert_eq!(
        test.token.balance(&test.seller),
        seller_balance_before + REFUND_AMOUNT as i128
    );

    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.status, ContractStatus::Completed);
}

#[test]
fn test_resolve_refund_dispute_non_admin() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    let refund_reason = String::from_str(&test.env, "Dispute reason");
    test.contract
        .request_refund(&contract_id, &test.buyer, &refund_reason);

    let result = test
        .contract
        .try_resolve_refund_dispute(&contract_id, &test.buyer, &true);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::AdminOnly));
}

#[test]
fn test_resolve_refund_dispute_not_requested() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    let result = test
        .contract
        .try_resolve_refund_dispute(&contract_id, &test.admin, &true);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::RefundNotRequested));
}

// Contract cancellation tests
#[test]
fn test_cancel_contract_success() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    let buyer_balance_before = test.token.balance(&test.buyer);

    test.contract.cancel_contract(&contract_id, &test.buyer);

    assert_eq!(
        test.token.balance(&test.buyer),
        buyer_balance_before + REFUND_AMOUNT as i128
    );

    let contract_data = test.contract.get_contract(&contract_id);
    assert_eq!(contract_data.status, ContractStatus::Cancelled);
    assert_eq!(contract_data.escrowed_amount, 0);
}

#[test]
fn test_cancel_contract_non_participant() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    let outsider = Address::generate(&test.env);
    let result = test.contract.try_cancel_contract(&contract_id, &outsider);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ParticipantOnly));
}

#[test]
fn test_cancel_completed_contract() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    test.contract.mark_delivered(&contract_id, &test.seller);
    test.contract.confirm_delivery(&contract_id, &test.buyer);

    let result = test.contract.try_cancel_contract(&contract_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::OperationNotAllowed));
}

#[test]
fn test_cancel_contract_after_delivery_marked() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    test.contract.mark_delivered(&contract_id, &test.seller);

    let result = test.contract.try_cancel_contract(&contract_id, &test.buyer);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::OperationNotAllowed));
}

// Edge case tests
#[test]
fn test_multiple_refund_attempts() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();

    let refund_reason = String::from_str(&test.env, "First refund");
    test.contract
        .request_refund(&contract_id, &test.buyer, &refund_reason);

    test.contract.process_automatic_refund(&contract_id);

    // Try to process again
    let result = test.contract.try_process_automatic_refund(&contract_id);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        Ok(ContractError::RefundConditionsNotMet)
    );
}

#[test]
fn test_invalid_refund_request_on_cancelled_contract() {
    let test = RefundTest::setup();
    let contract_id = test.create_refund_contract();

    test.contract.cancel_contract(&contract_id, &test.buyer);

    let refund_reason = String::from_str(&test.env, "Invalid request");
    let result = test
        .contract
        .try_request_refund(&contract_id, &test.buyer, &refund_reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::OperationNotAllowed));
}

#[test]
fn test_unauthorized_actions_protection() {
    let (test, contract_id) = RefundTest::setup_with_funded_contract();
    let unauthorized = Address::generate(&test.env);

    // Test various unauthorized access attempts
    let result = test
        .contract
        .try_mark_delivered(&contract_id, &unauthorized);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::SellerOnly));

    let result = test
        .contract
        .try_confirm_delivery(&contract_id, &unauthorized);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::BuyerOnly));

    let refund_reason = String::from_str(&test.env, "Unauthorized refund");
    let result = test
        .contract
        .try_request_refund(&contract_id, &unauthorized, &refund_reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Ok(ContractError::ParticipantOnly));
}

#[test]
fn test_get_user_contracts_pagination() {
    let test = RefundTest::setup();

    // Create multiple contracts
    let mut contract_ids = std::vec![];
    for _ in 0..5 {
        let contract_id = test.create_refund_contract();
        contract_ids.push(contract_id);
    }

    // Test pagination
    let first_page = test.contract.get_user_contracts(&test.buyer, &0, &3);
    assert_eq!(first_page.len(), 3);

    let second_page = test.contract.get_user_contracts(&test.buyer, &3, &3);
    assert_eq!(second_page.len(), 2);
}

#[test]
fn test_fund_contract_insufficient_balance() {
    let test = RefundTest::setup();

    // Create buyer with insufficient balance
    let poor_buyer = Address::generate(&test.env);

    let contract_id = test.contract.create_refund_contract(
        &poor_buyer,
        &test.seller,
        &test.token.address,
        &REFUND_AMOUNT,
        &REFUND_DEADLINE,
        &DELIVERY_DEADLINE,
        &test.refund_conditions,
    );

    // This should fail due to insufficient balance
    let result = test.contract.try_fund_contract(&contract_id, &poor_buyer);
    assert!(result.is_err());
    // The exact error will be a host error from the token contract
}
