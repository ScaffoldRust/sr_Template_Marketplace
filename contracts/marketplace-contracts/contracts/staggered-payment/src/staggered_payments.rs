use crate::storage_types::{DataKey, Milestone, Transactions, TIMEOUT};
use soroban_sdk::{contract, contractimpl, symbol_short, vec, Address, Env, Symbol, Vec};
use soroban_sdk::{log, token::TokenClient};

#[contract]
pub struct StaggeredPaymentContract;

#[contractimpl]
impl StaggeredPaymentContract {
    pub fn initialize(env: Env, payment_token: Address) {
        let storage = env.storage().persistent();
        let payment_token_option: Option<Address> = storage.get(&DataKey::PaymentToken);

        match payment_token_option {
            None => {
                storage.set(&DataKey::PaymentToken, &payment_token);
                storage.set(&DataKey::TransactionCount, &0u32);
                env.events()
                    .publish((symbol_short!("init"),), (payment_token,));
            }
            Some(_) => panic!("Contract already initialized"),
        }
    }

    pub fn create_transaction(
        env: Env,
        buyer: Address,
        seller: Address,
        total_amount: i128,
        milestone_percentages: Vec<i128>,
        milestone_descriptions: Vec<Symbol>,
    ) -> u32 {
        buyer.require_auth();
        assert!(total_amount > 0, "Total amount must be positive");
        assert!(
            milestone_percentages.len() == milestone_descriptions.len(),
            "Mismatching milestones and descriptions"
        );
        assert!(
            !milestone_percentages.is_empty(),
            "At least one milestone required"
        );

        // Verify percentages sum to 100
        let sum: i128 = milestone_percentages.iter().sum();
        assert!(sum == 100, "Milestone percentages must sum to 100");

        // Lock funds in escrow
        let token_client = TokenClient::new(&env, &Self::get_payment_token(&env));
        token_client.transfer(&buyer, &env.current_contract_address(), &total_amount);
        let tx_id = Self::increment_tx_count(&env);

        // Create milestones with calculated amounts
        let mut milestones = vec![&env];
        for i in 0..milestone_percentages.len() {
            let percentage = milestone_percentages
                .get(i)
                .expect("Failed to get milestone percentage");
            let description = milestone_descriptions
                .get(i)
                .expect("Failed to get milestone description");
            let amount = (total_amount * percentage) / 100;
            milestones.push_back(Milestone {
                id: i,
                amount,
                description,
                completed: false,
                approved: false,
                disputed: false,
            });
        }

        let transaction = Transactions {
            buyer: buyer.clone(),
            seller: seller.clone(),
            total_amount,
            milestones,
            created_at: env.ledger().timestamp(),
            is_active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Transaction(tx_id), &transaction);

        env.events().publish(
            (symbol_short!("new_tx"), tx_id),
            (buyer, seller, total_amount),
        );
        log!(&env, "Emitted new_tx event for tx_id: {}", tx_id);
        tx_id
    }

    pub fn release_funds(env: Env, tx_id: u32, milestone_id: u32, amount: i128) {
        let mut transaction = Self::get_transaction(&env, tx_id);
        assert!(transaction.is_active, "Transaction is not active");

        // Find milestone
        let mut milestones = vec![&env];
        let mut found = false;
        for milestone in transaction.milestones.iter() {
            if milestone.id == milestone_id {
                assert!(milestone.approved, "Milestone not approved");
                assert!(
                    milestone.amount >= amount,
                    "Release amount exceeds milestone amount"
                );
                milestones.push_back(Milestone {
                    id: milestone.id,
                    amount: milestone.amount - amount,
                    description: milestone.description.clone(),
                    completed: milestone.completed,
                    approved: milestone.approved,
                    disputed: milestone.disputed,
                });
                found = true;
            } else {
                milestones.push_back(milestone);
            }
        }
        assert!(found, "Milestone not found");

        transaction.milestones = milestones;

        // Release funds
        let token_client = TokenClient::new(&env, &Self::get_payment_token(&env));
        token_client.transfer(
            &env.current_contract_address(),
            &transaction.seller,
            &amount,
        );

        // Store updated transaction
        env.storage()
            .persistent()
            .set(&DataKey::Transaction(tx_id), &transaction);

        // Emit event
        env.events().publish(
            (symbol_short!("release"), tx_id, milestone_id),
            (transaction.seller.clone(), amount),
        );
    }

    pub fn submit_milestone(env: Env, tx_id: u32, milestone_id: u32) {
        let mut transaction = Self::get_transaction(&env, tx_id);
        transaction.seller.require_auth();
        assert!(transaction.is_active, "Transaction is not active");

        // Find and update milestone
        let mut milestones = vec![&env];
        let mut found = false;
        for milestone in transaction.milestones.iter() {
            if milestone.id == milestone_id {
                assert!(!milestone.completed, "Milestone already completed");
                assert!(!milestone.disputed, "Milestone is disputed");
                milestones.push_back(Milestone {
                    id: milestone.id,
                    amount: milestone.amount,
                    description: milestone.description,
                    completed: true,
                    approved: milestone.approved,
                    disputed: milestone.disputed,
                });
                found = true;
            } else {
                milestones.push_back(milestone);
            }
        }
        assert!(found, "Milestone not found");

        transaction.milestones = milestones;
        env.storage()
            .persistent()
            .set(&DataKey::Transaction(tx_id), &transaction);

        env.events().publish(
            (symbol_short!("submit"), tx_id, milestone_id),
            transaction.seller.clone(),
        );
    }

    pub fn approve_milestone(env: Env, tx_id: u32, milestone_id: u32) {
        let mut transaction = Self::get_transaction(&env, tx_id);
        transaction.buyer.require_auth();
        assert!(transaction.is_active, "Transaction is not active");

        // Find and update milestone
        let mut milestones = vec![&env];
        let mut found = false;
        let mut milestone_amount = 0;
        for milestone in transaction.milestones.iter() {
            if milestone.id == milestone_id {
                assert!(milestone.completed, "Milestone not yet completed");
                assert!(!milestone.approved, "Milestone already approved");
                assert!(!milestone.disputed, "Milestone is disputed");
                milestone_amount = milestone.amount;
                milestones.push_back(Milestone {
                    id: milestone.id,
                    amount: milestone.amount,
                    description: milestone.description,
                    completed: milestone.completed,
                    approved: true,
                    disputed: milestone.disputed,
                });
                found = true;
            } else {
                milestones.push_back(milestone);
            }
        }
        assert!(found, "Milestone not found");

        transaction.milestones = milestones;
        let all_approved = Self::all_milestones_approved(&transaction.milestones);
        if all_approved {
            transaction.is_active = false;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Transaction(tx_id), &transaction);

        env.events().publish(
            (symbol_short!("approve"), tx_id, milestone_id),
            (transaction.seller.clone(), milestone_amount),
        );

        if !transaction.is_active {
            env.events().publish(
                (symbol_short!("tx_done"), tx_id),
                (transaction.buyer.clone(), transaction.seller.clone()),
            );
        }
    }

    pub fn check_timeout(env: Env, tx_id: u32, milestone_id: u32) {
        let mut transaction = Self::get_transaction(&env, tx_id);
        assert!(transaction.is_active, "Transaction is not active");

        let current_time = env.ledger().timestamp();
        assert!(
            current_time >= transaction.created_at + TIMEOUT,
            "Timeout not reached"
        );

        // Find and update milestone
        let mut milestones = vec![&env];
        let mut found = false;
        let mut milestone_amount = 0;
        for milestone in transaction.milestones.iter() {
            if milestone.id == milestone_id {
                assert!(milestone.completed, "Milestone not yet completed");
                assert!(!milestone.approved, "Milestone already approved");
                assert!(!milestone.disputed, "Milestone is disputed");
                milestone_amount = milestone.amount;
                milestones.push_back(Milestone {
                    id: milestone.id,
                    amount: milestone.amount,
                    description: milestone.description,
                    completed: milestone.completed,
                    approved: true,
                    disputed: milestone.disputed,
                });
                found = true;
            } else {
                milestones.push_back(milestone);
            }
        }
        assert!(found, "Milestone not found");

        transaction.milestones = milestones;
        let all_approved = Self::all_milestones_approved(&transaction.milestones);
        if all_approved {
            transaction.is_active = false;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Transaction(tx_id), &transaction);

        env.events().publish(
            (symbol_short!("timeout"), tx_id, milestone_id),
            (transaction.seller.clone(), milestone_amount),
        );

        if !transaction.is_active {
            env.events().publish(
                (symbol_short!("tx_done"), tx_id),
                (transaction.buyer.clone(), transaction.seller.clone()),
            );
        }
    }

    pub fn dispute_milestone(env: Env, tx_id: u32, milestone_id: u32) {
        let mut transaction = Self::get_transaction(&env, tx_id);
        transaction.buyer.require_auth();
        assert!(transaction.is_active, "Transaction is not active");

        // Find and update milestone
        let mut milestones = vec![&env];
        let mut found = false;
        for milestone in transaction.milestones.iter() {
            if milestone.id == milestone_id {
                assert!(milestone.completed, "Milestone not yet completed");
                assert!(!milestone.approved, "Milestone already approved");
                assert!(!milestone.disputed, "Milestone already disputed");
                milestones.push_back(Milestone {
                    id: milestone.id,
                    amount: milestone.amount,
                    description: milestone.description,
                    completed: milestone.completed,
                    approved: milestone.approved,
                    disputed: true,
                });
                found = true;
            } else {
                milestones.push_back(milestone);
            }
        }
        assert!(found, "Milestone not found");

        transaction.milestones = milestones;
        env.storage()
            .persistent()
            .set(&DataKey::Transaction(tx_id), &transaction);

        env.events().publish(
            (symbol_short!("dispute"), tx_id, milestone_id),
            transaction.buyer.clone(),
        );
    }

    pub fn resolve_dispute(
        env: Env,
        tx_id: u32,
        milestone_id: u32,
        approve: bool,
        arbiter: Address,
    ) {
        arbiter.require_auth();
        let mut transaction = Self::get_transaction(&env, tx_id);
        assert!(transaction.is_active, "Transaction is not active");

        // Find and update milestone
        let mut milestones = vec![&env];
        let mut found = false;
        let mut milestone_amount = 0;
        for milestone in transaction.milestones.iter() {
            if milestone.id == milestone_id {
                assert!(milestone.disputed, "Milestone not disputed");
                milestone_amount = milestone.amount;
                milestones.push_back(Milestone {
                    id: milestone.id,
                    amount: milestone.amount,
                    description: milestone.description,
                    completed: milestone.completed,
                    approved: approve,
                    disputed: false,
                });
                found = true;
            } else {
                milestones.push_back(milestone);
            }
        }
        assert!(found, "Milestone not found");

        transaction.milestones = milestones;
        let transaction_complete = Self::is_transaction_complete(&transaction.milestones);
        if transaction_complete {
            transaction.is_active = false;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Transaction(tx_id), &transaction);

        if approve {
            env.events().publish(
                (symbol_short!("resolve"), tx_id, milestone_id),
                (transaction.seller.clone(), milestone_amount),
            );
        } else {
            env.events().publish(
                (symbol_short!("refund"), tx_id, milestone_id),
                (transaction.buyer.clone(), milestone_amount),
            );
        }

        if !transaction.is_active {
            env.events().publish(
                (symbol_short!("tx_done"), tx_id),
                (transaction.buyer.clone(), transaction.seller.clone()),
            );
        }
    }

    fn all_milestones_approved(milestones: &Vec<Milestone>) -> bool {
        milestones.iter().all(|m| m.approved)
    }

    fn is_transaction_complete(milestones: &Vec<Milestone>) -> bool {
        milestones.iter().all(|m| m.approved || !m.disputed)
    }

    fn get_transaction(env: &Env, tx_id: u32) -> Transactions {
        env.storage()
            .persistent()
            .get(&DataKey::Transaction(tx_id))
            .expect("Transaction not found")
    }

    fn increment_tx_count(env: &Env) -> u32 {
        let count = env
            .storage()
            .persistent()
            .get(&DataKey::TransactionCount)
            .unwrap_or(0u32);
        let new_count = count + 1;
        env.storage()
            .persistent()
            .set(&DataKey::TransactionCount, &new_count);
        new_count
    }

    fn get_payment_token(env: &Env) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::PaymentToken)
            .expect("Payment token not set")
    }
}
