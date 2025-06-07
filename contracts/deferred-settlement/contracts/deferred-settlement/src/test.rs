#![cfg(test)]

extern crate std;

use crate::types::{SettlementCondition, TransactionStatus};
use crate::{DeferredSettlementContract, DeferredSettlementContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::StellarAssetClient,
    Address, Env, Symbol,
};

#[test]
fn test_initialize_contract() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_address = env.register(DeferredSettlementContract, ());
    let contract_client = DeferredSettlementContractClient::new(&env, &contract_address);

    env.mock_all_auths();
    contract_client.initialize(&admin);

    assert_eq!(contract_client.get_total_transactions(), 0);
}

#[test]
fn test_create_transaction() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = StellarAssetClient::new(&env, &token_contract.address());

    let contract_address = env.register(DeferredSettlementContract, ());
    let contract_client = DeferredSettlementContractClient::new(&env, &contract_address);

    env.mock_all_auths();

    // Initialize contract and set token contract
    contract_client.initialize(&admin);
    contract_client.set_token_contract(&admin, &token_contract.address());

    // Setup transaction parameters
    let amount = 1000;
    let duration = 86400;
    let condition = Symbol::new(&env, "TimeBased");

    // Mint tokens to the buyer
    token_client.mint(&buyer, &amount);

    // Create transaction
    let transaction_id =
        contract_client.create_transaction(&buyer, &seller, &amount, &condition, &duration);

    // Verify transaction state
    let transaction = contract_client.get_transaction(&transaction_id);
    assert_eq!(transaction.id, transaction_id);
    assert_eq!(transaction.buyer, buyer);
    assert_eq!(transaction.seller, seller);
    assert_eq!(transaction.amount, amount);
    assert_eq!(transaction.status, TransactionStatus::Pending);
    assert_eq!(transaction.condition, SettlementCondition::TimeBased);
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_create_transaction_invalid_amount() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);

    let contract_address = env.register(DeferredSettlementContract, ());
    let contract_client = DeferredSettlementContractClient::new(&env, &contract_address);

    env.mock_all_auths();

    contract_client.initialize(&admin);
    contract_client.set_token_contract(&admin, &token_contract.address());

    let amount = 0;
    let duration = 86400;
    let condition = Symbol::new(&env, "TimeBased");

    contract_client.create_transaction(&buyer, &seller, &amount, &condition, &duration);
}

#[test]
fn test_verify_time_based_condition() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = StellarAssetClient::new(&env, &token_contract.address());

    let contract_address = env.register(DeferredSettlementContract, ());
    let contract_client = DeferredSettlementContractClient::new(&env, &contract_address);

    env.mock_all_auths();

    contract_client.initialize(&admin);
    contract_client.set_token_contract(&admin, &token_contract.address());

    let amount = 1000;
    let duration = 86400;
    let condition = Symbol::new(&env, "TimeBased");

    // Mint tokens to the buyer
    token_client.mint(&buyer, &amount);

    let transaction_id =
        contract_client.create_transaction(&buyer, &seller, &amount, &condition, &duration);

    // Fast forward ledger time
    env.ledger().with_mut(|l| l.timestamp += duration + 1);

    contract_client.verify_condition(&buyer, &transaction_id, &None);
    let transaction = contract_client.get_transaction(&transaction_id);
    assert_eq!(transaction.status, TransactionStatus::Completed);
}

#[test]
fn test_initiate_and_resolve_dispute() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = StellarAssetClient::new(&env, &token_contract.address());

    let contract_address = env.register(DeferredSettlementContract, ());
    let contract_client = DeferredSettlementContractClient::new(&env, &contract_address);

    env.mock_all_auths();

    contract_client.initialize(&admin);
    contract_client.set_token_contract(&admin, &token_contract.address());

    let amount = 1000;
    let duration = 86400;
    let condition = Symbol::new(&env, "BuyerApproval");

    // Mint tokens to the buyer
    token_client.mint(&buyer, &amount);

    let transaction_id =
        contract_client.create_transaction(&buyer, &seller, &amount, &condition, &duration);

    // Initiate dispute
    contract_client.initiate_dispute(&buyer, &transaction_id);

    let transaction = contract_client.get_transaction(&transaction_id);
    assert_eq!(transaction.status, TransactionStatus::Disputed);

    // Resolve dispute (release to seller)
    contract_client.resolve_dispute(&buyer, &transaction_id, &true);

    let transaction = contract_client.get_transaction(&transaction_id);
    assert_eq!(transaction.status, TransactionStatus::Completed);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_unauthorized_dispute() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = StellarAssetClient::new(&env, &token_contract.address());

    let contract_address = env.register(DeferredSettlementContract, ());
    let contract_client = DeferredSettlementContractClient::new(&env, &contract_address);

    env.mock_all_auths();

    contract_client.initialize(&admin);
    contract_client.set_token_contract(&admin, &token_contract.address());

    let amount = 1000;
    let duration = 86400;
    let condition = Symbol::new(&env, "BuyerApproval");

    // Mint tokens to the buyer
    token_client.mint(&buyer, &amount);

    let transaction_id =
        contract_client.create_transaction(&buyer, &seller, &amount, &condition, &duration);

    // Try to initiate dispute with unauthorized address
    contract_client.initiate_dispute(&unauthorized, &transaction_id);
}

#[test]
fn test_oracle_confirmation() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = StellarAssetClient::new(&env, &token_contract.address());

    let contract_address = env.register(DeferredSettlementContract, ());
    let contract_client = DeferredSettlementContractClient::new(&env, &contract_address);

    env.mock_all_auths();

    contract_client.initialize(&admin);
    contract_client.set_token_contract(&admin, &token_contract.address());

    let amount = 1000;
    let duration = 86400;
    let condition = Symbol::new(&env, "OracleConfirmation");

    // Mint tokens to the buyer
    token_client.mint(&buyer, &amount);

    let transaction_id =
        contract_client.create_transaction(&buyer, &seller, &amount, &condition, &duration);

    contract_client.verify_condition(&admin, &transaction_id, &Some(true));

    let transaction = contract_client.get_transaction(&transaction_id);
    assert_eq!(transaction.status, TransactionStatus::Completed);
}
