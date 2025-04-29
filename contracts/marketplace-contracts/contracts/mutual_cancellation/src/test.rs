#![cfg(test)]

use crate::{
    contract::{MutualCancellation, MutualCancellationClient},
    storage_types::{CancellationStatus, Transaction},
};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events, Ledger, LedgerInfo},
    token, Address, BytesN, Env, IntoVal, Symbol,
};

// Helper to create a token contract for testing
fn create_token_contract(e: &Env, admin: &Address) -> (Address, token::Client) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (contract_address.clone(), token::Client::new(e, &contract_address))
}

// Helper to create test users with initial token balance
fn create_user(e: &Env, token: &token::Client, admin: &Address, amount: i128) -> Address {
    let user = Address::generate(e);
    token.mint(&admin, &user, &amount);
    user
}

// Helper to set up a standard test environment
fn setup_test() -> (
    Env,
    MutualCancellationClient,
    Address,
    Address,
    Address,
    token::Client,
) {
    let env = Env::default();
    env.mock_all_auths();

    // Set up the token
    let admin = Address::generate(&env);
    let (token_address, token_client) = create_token_contract(&env, &admin);

    // Create buyer and seller with funds
    let buyer = create_user(&env, &token_client, &admin, 10000);
    let seller = Address::generate(&env);

    // Set up the mutual cancellation contract
    let contract_id = env.register_contract(None, MutualCancellation);
    let client = MutualCancellationClient::new(&env, &contract_id);

    // Initialize the contract with a 7-day response window (simulating 7 days in seconds)
    client.initialize(&(7 * 24 * 60 * 60));

    (env, client, buyer, seller, token_address, token_client)
}

// Helper to create a test transaction
fn create_test_transaction(
    client: &MutualCancellationClient,
    buyer: &Address,
    seller: &Address,
    token_address: &Address,
    amount: i128,
) -> u64 {
    client.create_transaction(buyer, seller, token_address, &amount)
}

// Test successful creation of a transaction
#[test]
fn test_create_transaction() {
    let (env, client, buyer, seller, token_address, token_client) = setup_test();

    // Initial balance of buyer and contract
    let contract_address = env.current_contract_address();
    let initial_buyer_balance = token_client.balance(&buyer);
    let initial_contract_balance = token_client.balance(&contract_address);
    
    // Create a transaction with 1000 tokens
    let amount = 1000;
    let tx_id = create_test_transaction(&client, &buyer, &seller, &token_address, amount);
    
    // Verify transaction exists and has correct details
    let tx = client.get_transaction(&tx_id).expect("Transaction should exist");
    assert_eq!(tx.id, tx_id);
    assert_eq!(tx.buyer, buyer);
    assert_eq!(tx.seller, seller);
    assert_eq!(tx.token, token_address);
    assert_eq!(tx.amount, amount);
    assert_eq!(tx.status, CancellationStatus::None);
    
    // Verify funds are transferred to the contract
    let final_buyer_balance = token_client.balance(&buyer);
    let final_contract_balance = token_client.balance(&contract_address);
    assert_eq!(final_buyer_balance, initial_buyer_balance - amount);
    assert_eq!(final_contract_balance, initial_contract_balance + amount);
    
    // Verify transaction shows up in buyer's and seller's transactions
    let buyer_txs = client.get_buyer_transactions(&buyer);
    let seller_txs = client.get_seller_transactions(&seller);
    assert_eq!(buyer_txs.len(), 1);
    assert_eq!(seller_txs.len(), 1);
    assert_eq!(buyer_txs.get(0).unwrap().id, tx_id);
    assert_eq!(seller_txs.get(0).unwrap().id, tx_id);
    
    // Verify event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    
    let topics = events[0].topics.clone();
    assert_eq!(topics.get(0).unwrap(), Symbol::new(&env, "tx_created").into_val(&env));
    assert_eq!(topics.get(1).unwrap(), tx_id.into_val(&env));
    assert_eq!(topics.get(2).unwrap(), buyer.into_val(&env));
    assert_eq!(topics.get(3).unwrap(), seller.into_val(&env));
}

// Test proposal of cancellation by the buyer
#[test]
fn test_buyer_proposes_cancellation() {
    let (env, client, buyer, seller, token_address, _token_client) = setup_test();
    
    // Create a transaction
    let amount = 1000;
    let tx_id = create_test_transaction(&client, &buyer, &seller, &token_address, amount);
    
    // Buyer proposes cancellation
    client.buyer_propose_cancellation(&tx_id);
    
    // Verify transaction status is updated
    let tx = client.get_transaction(&tx_id).expect("Transaction should exist");
    assert_eq!(tx.status, CancellationStatus::ProposedByBuyer);
    assert!(tx.proposal_timestamp > 0);
    
    // Verify event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 2); // Transaction creation + proposal
    
    let topics = events[1].topics.clone();
    assert_eq!(topics.get(0).unwrap(), Symbol::new(&env, "can_prop").into_val(&env));
    assert_eq!(topics.get(1).unwrap(), tx_id.into_val(&env));
    assert_eq!(topics.get(2).unwrap(), buyer.into_val(&env));
}

// Test proposal of cancellation by the seller
#[test]
fn test_seller_proposes_cancellation() {
    let (env, client, buyer, seller, token_address, _token_client) = setup_test();
    
    // Create a transaction
    let amount = 1000;
    let tx_id = create_test_transaction(&client, &buyer, &seller, &token_address, amount);
    
    // Seller proposes cancellation
    client.seller_propose_cancellation(&tx_id);
    
    // Verify transaction status is updated
    let tx = client.get_transaction(&tx_id).expect("Transaction should exist");
    assert_eq!(tx.status, CancellationStatus::ProposedBySeller);
    assert!(tx.proposal_timestamp > 0);
    
    // Verify event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 2); // Transaction creation + proposal
    
    let topics = events[1].topics.clone();
    assert_eq!(topics.get(0).unwrap(), Symbol::new(&env, "can_prop").into_val(&env));
    assert_eq!(topics.get(1).unwrap(), tx_id.into_val(&env));
    assert_eq!(topics.get(2).unwrap(), seller.into_val(&env));
}

// Test successful mutual cancellation (buyer proposes, seller agrees)
#[test]
fn test_buyer_proposes_seller_agrees() {
    let (env, client, buyer, seller, token_address, token_client) = setup_test();
    
    // Create a transaction
    let amount = 1000;
    let tx_id = create_test_transaction(&client, &buyer, &seller, &token_address, amount);
    
    // Buyer proposes cancellation
    client.buyer_propose_cancellation(&tx_id);
    
    // Record balances before cancellation agreement
    let contract_address = env.current_contract_address();
    let buyer_balance_before = token_client.balance(&buyer);
    let contract_balance_before = token_client.balance(&contract_address);
    
    // Seller agrees to cancellation
    env.set_invoker(seller.clone());
    client.agree_to_cancellation(&tx_id);
    
    // Verify transaction status is updated
    let tx = client.get_transaction(&tx_id).expect("Transaction should exist");
    assert_eq!(tx.status, CancellationStatus::Completed);
    
    // Verify funds are returned to buyer
    let buyer_balance_after = token_client.balance(&buyer);
    let contract_balance_after = token_client.balance(&contract_address);
    assert_eq!(buyer_balance_after, buyer_balance_before + amount);
    assert_eq!(contract_balance_after, contract_balance_before - amount);
    
    // Verify event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 3); // Transaction creation + proposal + agreement
    
    let topics = events[2].topics.clone();
    assert_eq!(topics.get(0).unwrap(), Symbol::new(&env, "can_agree").into_val(&env));
    assert_eq!(topics.get(1).unwrap(), tx_id.into_val(&env));
}

// Test successful mutual cancellation (seller proposes, buyer agrees)
#[test]
fn test_seller_proposes_buyer_agrees() {
    let (env, client, buyer, seller, token_address, token_client) = setup_test();
    
    // Create a transaction
    let amount = 1000;
    let tx_id = create_test_transaction(&client, &buyer, &seller, &token_address, amount);
    
    // Seller proposes cancellation
    client.seller_propose_cancellation(&tx_id);
    
    // Record balances before cancellation agreement
    let contract_address = env.current_contract_address();
    let buyer_balance_before = token_client.balance(&buyer);
    let contract_balance_before = token_client.balance(&contract_address);
    
    // Buyer agrees to cancellation
    env.set_invoker(buyer.clone());
    client.agree_to_cancellation(&tx_id);
    
    // Verify transaction status is updated
    let tx = client.get_transaction(&tx_id).expect("Transaction should exist");
    assert_eq!(tx.status, CancellationStatus::Completed);
    
    // Verify funds are returned to buyer
    let buyer_balance_after = token_client.balance(&buyer);
    let contract_balance_after = token_client.balance(&contract_address);
    assert_eq!(buyer_balance_after, buyer_balance_before + amount);
    assert_eq!(contract_balance_after, contract_balance_before - amount);
    
    // Verify event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 3); // Transaction creation + proposal + agreement
    
    let topics = events[2].topics.clone();
    assert_eq!(topics.get(0).unwrap(), Symbol::new(&env, "can_agree").into_val(&env));
}

// Test proposal expiration
#[test]
fn test_proposal_expires() {
    let (mut env, client, buyer, seller, token_address, _token_client) = setup_test();
    
    // Create a transaction
    let amount = 1000;
    let tx_id = create_test_transaction(&client, &buyer, &seller, &token_address, amount);
    
    // Buyer proposes cancellation
    client.buyer_propose_cancellation(&tx_id);
    
    // Advance ledger time beyond response window (7 days + 1 second)
    let response_window = client.get_response_window();
    env.ledger().with_mut(|li: &mut LedgerInfo| {
        li.timestamp += response_window + 1;
    });
    
    // Check if proposal has expired
    let is_expired = client.check_cancellation_expiry(&tx_id);
    assert!(is_expired);
    
    // Reset expired proposal
    client.reset_expired_proposal(&tx_id);
    
    // Verify transaction status is reset
    let tx = client.get_transaction(&tx_id).expect("Transaction should exist");
    assert_eq!(tx.status, CancellationStatus::None);
    
    // Verify event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 3); // Transaction creation + proposal + expiry
    
    let topics = events[2].topics.clone();
    assert_eq!(topics.get(0).unwrap(), Symbol::new(&env, "can_exp").into_val(&env));
}

// Test that third party cannot propose cancellation (testing buyer proposal)
#[test]
#[should_panic(expected = "Unauthorized")] // Expect require_auth panic
fn test_third_party_cannot_propose_buyer() {
    let (env, client, buyer, seller, token_address, _token_client) = setup_test();
    
    // Create a transaction
    let amount = 1000;
    let tx_id = create_test_transaction(&client, &buyer, &seller, &token_address, amount);
    
    // Third party tries to propose cancellation as buyer
    let third_party = Address::generate(&env);
    env.set_invoker(third_party);
    client.buyer_propose_cancellation(&tx_id);
}

// Test that third party cannot propose cancellation (testing seller proposal)
#[test]
#[should_panic(expected = "Unauthorized")] // Expect require_auth panic
fn test_third_party_cannot_propose_seller() {
    let (env, client, buyer, seller, token_address, _token_client) = setup_test();
    
    // Create a transaction
    let amount = 1000;
    let tx_id = create_test_transaction(&client, &buyer, &seller, &token_address, amount);
    
    // Third party tries to propose cancellation as seller
    let third_party = Address::generate(&env);
    env.set_invoker(third_party);
    client.seller_propose_cancellation(&tx_id);
}

// Test that wrong party cannot agree to cancellation
#[test]
#[should_panic(expected = "Unauthorized")] // Expect require_auth panic
fn test_wrong_party_cannot_agree() {
    let (env, client, buyer, seller, token_address, _token_client) = setup_test();
    
    // Create a transaction
    let amount = 1000;
    let tx_id = create_test_transaction(&client, &buyer, &seller, &token_address, amount);
    
    // Buyer proposes cancellation
    client.buyer_propose_cancellation(&tx_id);
    
    // Third party tries to agree (should require seller auth)
    let third_party = Address::generate(&env);
    env.set_invoker(third_party);
    client.agree_to_cancellation(&tx_id);
}

// Test that buyer cannot agree to their own proposal
#[test]
#[should_panic(expected = "Unauthorized")] // Expect require_auth panic (needs seller auth)
fn test_buyer_cannot_agree_to_own_proposal() {
    let (env, client, buyer, seller, token_address, _token_client) = setup_test();
    
    // Create a transaction
    let amount = 1000;
    let tx_id = create_test_transaction(&client, &buyer, &seller, &token_address, amount);
    
    // Buyer proposes cancellation
    client.buyer_propose_cancellation(&tx_id);
    
    // Buyer tries to agree to their own proposal (should require seller auth)
    client.agree_to_cancellation(&tx_id);
}

// Test that seller cannot agree to their own proposal
#[test]
#[should_panic(expected = "Unauthorized")] // Expect require_auth panic (needs buyer auth)
fn test_seller_cannot_agree_to_own_proposal() {
    let (env, client, buyer, seller, token_address, _token_client) = setup_test();
    
    // Create a transaction
    let amount = 1000;
    let tx_id = create_test_transaction(&client, &buyer, &seller, &token_address, amount);
    
    // Seller proposes cancellation
    client.seller_propose_cancellation(&tx_id);
    
    // Seller tries to agree to their own proposal (should require buyer auth)
    client.agree_to_cancellation(&tx_id);
}

// Test that cannot agree after proposal expiration
#[test]
#[should_panic(expected = "Cancellation proposal has expired")]
fn test_cannot_agree_after_expiration() {
    let (mut env, client, buyer, seller, token_address, _token_client) = setup_test();
    
    // Create a transaction
    let amount = 1000;
    let tx_id = create_test_transaction(&client, &buyer, &seller, &token_address, amount);
    
    // Buyer proposes cancellation
    client.buyer_propose_cancellation(&tx_id);
    
    // Advance ledger time beyond response window
    let response_window = client.get_response_window();
    env.ledger().with_mut(|li: &mut LedgerInfo| {
        li.timestamp += response_window + 1;
    });
    
    // Seller tries to agree after expiration
    env.set_invoker(seller.clone());
    client.agree_to_cancellation(&tx_id);
}

// Test that new proposal can be made after previous one expires
#[test]
fn test_new_proposal_after_expiration() {
    let (mut env, client, buyer, seller, token_address, _token_client) = setup_test();
    
    // Create a transaction
    let amount = 1000;
    let tx_id = create_test_transaction(&client, &buyer, &seller, &token_address, amount);
    
    // Buyer proposes cancellation
    client.buyer_propose_cancellation(&tx_id);
    
    // Advance ledger time beyond response window
    let response_window = client.get_response_window();
    env.ledger().with_mut(|li: &mut LedgerInfo| {
        li.timestamp += response_window + 1;
    });
    
    // Seller makes a new proposal (should reset status internally if expired)
    client.seller_propose_cancellation(&tx_id);
    
    // Verify transaction status is updated to seller's proposal
    let tx = client.get_transaction(&tx_id).expect("Transaction should exist");
    assert_eq!(tx.status, CancellationStatus::ProposedBySeller);
} 