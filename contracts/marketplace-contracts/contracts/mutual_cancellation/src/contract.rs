use crate::events::Events;
use crate::storage_types::{CancellationStatus, DataKey, Transaction, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD, TRANSACTION_BUMP_AMOUNT, TRANSACTION_LIFETIME_THRESHOLD};
use soroban_sdk::{contract, contractimpl, token, vec, Address, Env, Vec};

#[contract]
pub struct MutualCancellation;

// Helper functions for storage operations
fn get_transaction(env: &Env, id: u64) -> Option<Transaction> {
    let key = DataKey::Transaction(id);
    env.storage().instance().get(&key)
}

fn save_transaction(env: &Env, transaction: &Transaction) {
    let key = DataKey::Transaction(transaction.id);
    env.storage().instance().set(&key, transaction);
    env.storage()
        .instance()
        .extend_ttl(TRANSACTION_LIFETIME_THRESHOLD, TRANSACTION_BUMP_AMOUNT);
}

fn get_transaction_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::TransactionCounter)
        .unwrap_or(0)
}

fn increment_transaction_counter(env: &Env) -> u64 {
    let counter = get_transaction_counter(env) + 1;
    env.storage()
        .instance()
        .set(&DataKey::TransactionCounter, &counter);
    counter
}

fn get_buyer_transactions(env: &Env, buyer: &Address) -> Vec<u64> {
    env.storage()
        .instance()
        .get(&DataKey::BuyerTransactions(buyer.clone()))
        .unwrap_or_else(|| vec![env])
}

fn add_buyer_transaction(env: &Env, buyer: &Address, transaction_id: u64) {
    let mut transactions = get_buyer_transactions(env, buyer);
    transactions.push_back(transaction_id);
    env.storage()
        .instance()
        .set(&DataKey::BuyerTransactions(buyer.clone()), &transactions);
}

fn get_seller_transactions(env: &Env, seller: &Address) -> Vec<u64> {
    env.storage()
        .instance()
        .get(&DataKey::SellerTransactions(seller.clone()))
        .unwrap_or_else(|| vec![env])
}

fn add_seller_transaction(env: &Env, seller: &Address, transaction_id: u64) {
    let mut transactions = get_seller_transactions(env, seller);
    transactions.push_back(transaction_id);
    env.storage()
        .instance()
        .set(&DataKey::SellerTransactions(seller.clone()), &transactions);
}

fn get_response_window(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::ResponseWindow)
        .unwrap_or(7 * 24 * 60 * 60) // Default: 7 days in seconds
}

#[contractimpl]
impl MutualCancellation {
    // Initialize the contract with a default response window for cancellation proposals
    pub fn initialize(env: Env, response_window: u64) {
        env.storage()
            .instance()
            .set(&DataKey::ResponseWindow, &response_window);
        
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
    }

    // Create a new transaction in escrow
    pub fn create_transaction(
        env: Env,
        buyer: Address,
        seller: Address,
        token: Address,
        amount: i128,
    ) -> u64 {
        // Validate inputs
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Authenticate the buyer who is creating the transaction
        buyer.require_auth();

        // Transfer funds from buyer to the contract
        token::Client::new(&env, &token).transfer(
            &buyer,
            &env.current_contract_address(),
            &amount,
        );

        // Generate a new transaction ID
        let id = increment_transaction_counter(&env);

        // Create the transaction record
        let transaction = Transaction {
            id,
            buyer: buyer.clone(),
            seller: seller.clone(),
            token,
            amount,
            status: CancellationStatus::None,
            proposal_timestamp: 0,
            response_window: get_response_window(&env),
        };

        // Store the transaction
        save_transaction(&env, &transaction);
        add_buyer_transaction(&env, &buyer, id);
        add_seller_transaction(&env, &seller, id);

        // Emit event
        Events::new(&env).transaction_created(&transaction);

        id
    }

    // Get details of a specific transaction
    pub fn get_transaction(env: Env, id: u64) -> Option<Transaction> {
        get_transaction(&env, id)
    }

    // Get all transactions associated with a buyer
    pub fn get_buyer_transactions(env: Env, buyer: Address) -> Vec<Transaction> {
        let transaction_ids = get_buyer_transactions(&env, &buyer);
        let mut transactions = vec![&env];
        
        for id in transaction_ids.iter() {
            if let Some(tx) = get_transaction(&env, id) {
                transactions.push_back(tx);
            }
        }
        
        transactions
    }

    // Get all transactions associated with a seller
    pub fn get_seller_transactions(env: Env, seller: Address) -> Vec<Transaction> {
        let transaction_ids = get_seller_transactions(&env, &seller);
        let mut transactions = vec![&env];
        
        for id in transaction_ids.iter() {
            if let Some(tx) = get_transaction(&env, id) {
                transactions.push_back(tx);
            }
        }
        
        transactions
    }

    // Buyer proposes cancellation of a transaction
    pub fn buyer_propose_cancellation(env: Env, id: u64) {
        // Ensure the transaction exists
        let transaction_opt = get_transaction(&env, id);
        if transaction_opt.is_none() {
            panic!("Transaction not found");
        }
        
        let mut transaction = transaction_opt.unwrap();
        
        // Check if cancellation is already completed
        if transaction.status == CancellationStatus::Completed {
            panic!("Transaction already cancelled");
        }
        
        // Check if there's already a proposal in place
        if transaction.status != CancellationStatus::None {
            // Check if the proposal has expired
            let current_time = env.ledger().timestamp();
            if current_time > transaction.proposal_timestamp + transaction.response_window {
                // Reset expired proposal
                transaction.status = CancellationStatus::None;
            } else {
                panic!("Cancellation already proposed");
            }
        }
        
        // Require buyer auth
        transaction.buyer.require_auth();
                
        // Update transaction status
        transaction.status = CancellationStatus::ProposedByBuyer;
        transaction.proposal_timestamp = env.ledger().timestamp();
        
        // Save updated transaction
        save_transaction(&env, &transaction);
        
        // Emit event
        Events::new(&env).cancellation_proposed(&transaction);
    }

    // Seller proposes cancellation of a transaction
    pub fn seller_propose_cancellation(env: Env, id: u64) {
        // Ensure the transaction exists
        let transaction_opt = get_transaction(&env, id);
        if transaction_opt.is_none() {
            panic!("Transaction not found");
        }
        
        let mut transaction = transaction_opt.unwrap();
        
        // Check if cancellation is already completed
        if transaction.status == CancellationStatus::Completed {
            panic!("Transaction already cancelled");
        }
        
        // Check if there's already a proposal in place
        if transaction.status != CancellationStatus::None {
            // Check if the proposal has expired
            let current_time = env.ledger().timestamp();
            if current_time > transaction.proposal_timestamp + transaction.response_window {
                // Reset expired proposal
                transaction.status = CancellationStatus::None;
            } else {
                panic!("Cancellation already proposed");
            }
        }
        
        // Require seller auth
        transaction.seller.require_auth();
                
        // Update transaction status
        transaction.status = CancellationStatus::ProposedBySeller;
        transaction.proposal_timestamp = env.ledger().timestamp();
        
        // Save updated transaction
        save_transaction(&env, &transaction);
        
        // Emit event
        Events::new(&env).cancellation_proposed(&transaction);
    }

    // Agree to a cancellation proposal
    pub fn agree_to_cancellation(env: Env, id: u64) {
        // Ensure the transaction exists
        let transaction_opt = get_transaction(&env, id);
        if transaction_opt.is_none() {
            panic!("Transaction not found");
        }
        
        let mut transaction = transaction_opt.unwrap();
        
        // Check if cancellation is already completed
        if transaction.status == CancellationStatus::Completed {
            panic!("Transaction already cancelled");
        }
        
        // Check if there's a proposal in place
        if transaction.status == CancellationStatus::None {
            panic!("No cancellation proposal to agree to");
        }
        
        // Check if the proposal has expired
        let current_time = env.ledger().timestamp();
        if current_time > transaction.proposal_timestamp + transaction.response_window {
            // Reset proposal and notify
            transaction.status = CancellationStatus::None;
            save_transaction(&env, &transaction);
            Events::new(&env).cancellation_expired(&transaction);
            panic!("Cancellation proposal has expired");
        }
        
        // Verify the caller is the correct counterparty and require their auth
        match transaction.status {
            CancellationStatus::ProposedByBuyer => {
                transaction.seller.require_auth(); // Require seller auth
            },
            CancellationStatus::ProposedBySeller => {
                 transaction.buyer.require_auth(); // Require buyer auth
            },
            _ => panic!("Invalid cancellation status"),
        }
        
        // Return funds to buyer
        token::Client::new(&env, &transaction.token).transfer(
            &env.current_contract_address(),
            &transaction.buyer,
            &transaction.amount,
        );
        
        // Update transaction status
        transaction.status = CancellationStatus::Completed;
        
        // Save updated transaction
        save_transaction(&env, &transaction);
        
        // Emit event
        Events::new(&env).cancellation_agreed(&transaction);
    }

    // Check if a cancellation proposal has expired
    pub fn check_cancellation_expiry(env: Env, id: u64) -> bool {
        // Ensure the transaction exists
        let transaction_opt = get_transaction(&env, id);
        if transaction_opt.is_none() {
            panic!("Transaction not found");
        }
        
        let transaction = transaction_opt.unwrap();
        
        // Check if there's a proposal in place
        if transaction.status == CancellationStatus::None || transaction.status == CancellationStatus::Completed {
            return false;
        }
        
        // Check if the proposal has expired
        let current_time = env.ledger().timestamp();
        if current_time > transaction.proposal_timestamp + transaction.response_window {
            return true;
        }
        
        false
    }

    // Reset an expired cancellation proposal
    pub fn reset_expired_proposal(env: Env, id: u64) {
        // Ensure the transaction exists
        let transaction_opt = get_transaction(&env, id);
        if transaction_opt.is_none() {
            panic!("Transaction not found");
        }
        
        let mut transaction = transaction_opt.unwrap();
        
        // Check if there's a proposal in place
        if transaction.status == CancellationStatus::None || transaction.status == CancellationStatus::Completed {
            panic!("No cancellation proposal to reset");
        }
        
        // Check if the proposal has expired
        let current_time = env.ledger().timestamp();
        if current_time <= transaction.proposal_timestamp + transaction.response_window {
            panic!("Cancellation proposal has not expired");
        }
        
        // Reset proposal
        transaction.status = CancellationStatus::None;
        
        // Save updated transaction
        save_transaction(&env, &transaction);
        
        // Emit event
        Events::new(&env).cancellation_expired(&transaction);
    }

    // Get the current response window setting
    pub fn get_response_window(env: Env) -> u64 {
        get_response_window(&env)
    }
} 