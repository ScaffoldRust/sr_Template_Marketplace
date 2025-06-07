#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    token::{StellarAssetClient, TokenClient},
    Address, Env, IntoVal,
};

use crate::datatypes::EscrowStatus;
use crate::escrow::{EscrowContract, EscrowContractClient};

fn create_client<'a>(env: &'a Env) -> EscrowContractClient<'a> {
    let contract_id = env.register_contract(None, EscrowContract);
    EscrowContractClient::new(env, &contract_id)
}

#[test]
fn test_initialize_success() {
    let env = Env::default();
    let client = create_client(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let mediator = Some(Address::generate(&env));
    let token = Address::generate(&env);

    let result = client.initialize(&buyer, &seller, &mediator, &token, &1000, &2, &1_000_000);

    assert_eq!(result, ());
    let state = client.get_state();
    assert_eq!(state.buyer, buyer);
    assert_eq!(state.seller, seller);
    assert_eq!(state.mediator, mediator);
    assert_eq!(state.amount, 1000);
    assert_eq!(state.required_approvals, 2);
    assert_eq!(state.status, EscrowStatus::Initialized);
}
#[test]
fn test_deposit_and_approve_flow() {
    let env = Env::default();
    env.mock_all_auths();

    // Create actors
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let approver1 = Some(Address::generate(&env));

    // Deploy mock token contract and fund buyer
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_address = sac.address();
    let token = TokenClient::new(&env, &token_address);
    let token_sac = StellarAssetClient::new(&env, &token_address);

    token_sac.mint(&buyer, &1000);

    // Deploy and initialize escrow
    let escrow_address = env.register_contract(None, EscrowContract);
    let escrow = EscrowContractClient::new(&env, &escrow_address);

    escrow.initialize(
        &buyer,
        &seller,
        &approver1,
        &token_address,
        &1000,
        &2,
        &1_000_000,
    );

    escrow.deposit();

    // Approvals
    env.mock_auths(&[MockAuth {
        address: &buyer,
        invoke: &MockAuthInvoke {
            contract: &escrow_address,
            fn_name: "approve",
            args: (buyer.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    escrow.approve(&buyer);

    env.mock_auths(&[MockAuth {
        address: &seller,
        invoke: &MockAuthInvoke {
            contract: &escrow_address,
            fn_name: "approve",
            args: (seller.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    escrow.approve(&seller);

    // Release funds
    escrow.release();

    // Check balance
    assert_eq!(token.balance(&seller), 1000);
}

#[test]
fn test_release_funds() {
    let env = Env::default();
    env.mock_all_auths();

    // Create actors
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let issuer = Address::generate(&env);

    // Create token
    let sac = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_address = sac.address();
    let token = TokenClient::new(&env, &token_address);
    let token_sac = StellarAssetClient::new(&env, &token_address);

    token_sac.mint(&buyer, &1000);

    // Register escrow contract
    let escrow_address = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &escrow_address);

    // Initialize escrow
    client.initialize(
        &buyer,
        &seller,
        &None,
        &token_address,
        &1000,
        &2,
        &1_000_000,
    );

    client.deposit();

    // Approvals
    for addr in [&buyer, &seller] {
        client.approve(addr);
    }

    // Release funds
    client.release();

    let state = client.get_state();
    assert_eq!(state.status, EscrowStatus::Released);
    assert_eq!(token.balance(&seller), 1000);
}

#[test]
fn test_refund_after_deadline() {
    let env = Env::default();
    env.mock_all_auths();

    // Create actors
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let issuer = Address::generate(&env);

    // Create token
    let sac = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_address = sac.address();
    let token = TokenClient::new(&env, &token_address);
    let token_sac = StellarAssetClient::new(&env, &token_address);

    token_sac.mint(&buyer, &1000);

    // Register escrow contract
    let escrow_address = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &escrow_address);

    let deadline = env.ledger().timestamp();

    // Initialize escrow
    client.initialize(&buyer, &seller, &None, &token_address, &1000, &2, &deadline);

    client.deposit();

    // Refund
    client.refund();

    let state = client.get_state();
    assert_eq!(state.status, EscrowStatus::Refunded);
    assert_eq!(token.balance(&buyer), 1000);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_refund_fails_before_deadline() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let issuer = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_address = sac.address();
    let token_sac = StellarAssetClient::new(&env, &token_address);
    token_sac.mint(&buyer, &1000);

    let escrow = create_client(&env);
    let deadline = env.ledger().timestamp() + 100;

    escrow.initialize(&buyer, &seller, &None, &token_address, &1000, &2, &deadline);
    escrow.deposit();

    escrow.refund();
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_release_fails_if_not_enough_approvals() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let issuer = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_address = sac.address();
    let token_sac = StellarAssetClient::new(&env, &token_address);
    token_sac.mint(&buyer, &1000);

    let escrow = create_client(&env);
    escrow.initialize(&buyer, &seller, &None, &token_address, &1000, &2, &1000_000);
    escrow.deposit();

    // Only one approval
    escrow.approve(&buyer);

    escrow.release();
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_initialize_fails_if_already_initialized() {
    let env = Env::default();
    let client = create_client(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = Address::generate(&env);

    // First initialization
    client.initialize(&buyer, &seller, &None, &token, &1000, &2, &1000_000);

    // Second initialization should fail

    client.initialize(&buyer, &seller, &None, &token, &1000, &2, &1000_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_approve_fails_if_unauthorized_party() {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let random = Address::generate(&env);
    let issuer = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_address = sac.address();
    let token_sac = StellarAssetClient::new(&env, &token_address);
    token_sac.mint(&buyer, &1000);

    let escrow = create_client(&env);
    escrow.initialize(&buyer, &seller, &None, &token_address, &1000, &2, &1000_000);
    escrow.deposit();

    escrow.approve(&random);
}

#[test]
fn test_get_state_returns_correct_status() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup addresses
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let issuer = Address::generate(&env);

    // Create token and mint to buyer
    let sac = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_address = sac.address();
    let token_sac = StellarAssetClient::new(&env, &token_address);
    token_sac.mint(&buyer, &1000);

    // Register and initialize the contract
    let client = create_client(&env);
    client.initialize(&buyer, &seller, &None, &token_address, &1000, &2, &1000_000);

    // Assert state is Initialized
    let state = client.get_state();
    assert_eq!(state.status, EscrowStatus::Initialized);

    // Deposit and assert state
    client.deposit();
    let state = client.get_state();
    assert_eq!(state.status, EscrowStatus::Deposited);

    // Approve by buyer and seller
    client.approve(&buyer);
    client.approve(&seller);

    // Release funds
    client.release();
    let state = client.get_state();
    assert_eq!(state.status, EscrowStatus::Released);
}
