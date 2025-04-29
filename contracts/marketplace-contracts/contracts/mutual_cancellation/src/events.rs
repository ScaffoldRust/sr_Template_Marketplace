use crate::storage_types::{Transaction, CancellationStatus};
use soroban_sdk::{symbol_short, Env};

pub struct Events {
    env: Env,
}

impl Events {
    #[inline(always)]
    pub fn new(env: &Env) -> Events {
        Events { env: env.clone() }
    }

    // Emit an event when a transaction is created
    pub fn transaction_created(&self, transaction: &Transaction) {
        let topics = (
            symbol_short!("tx_create"),
            transaction.id,
            transaction.buyer.clone(),
            transaction.seller.clone(),
        );
        self.env.events().publish(
            topics, 
            (transaction.token.clone(), transaction.amount)
        );
    }

    // Emit an event when a cancellation is proposed
    pub fn cancellation_proposed(&self, transaction: &Transaction) {
        let proposer = match transaction.status {
            CancellationStatus::ProposedByBuyer => transaction.buyer.clone(),
            CancellationStatus::ProposedBySeller => transaction.seller.clone(),
            _ => panic!("Invalid cancellation status for proposal event"),
        };

        let topics = (
            symbol_short!("can_prop"),
            transaction.id,
            proposer,
        );
        self.env.events().publish(
            topics, 
            (transaction.proposal_timestamp, transaction.response_window)
        );
    }

    // Emit an event when a cancellation is agreed to
    pub fn cancellation_agreed(&self, transaction: &Transaction) {
        let topics = (
            symbol_short!("can_agree"),
            transaction.id,
            transaction.buyer.clone(),
            transaction.seller.clone(),
        );
        self.env.events().publish(
            topics, 
            transaction.amount
        );
    }

    // Emit an event when a cancellation proposal expires
    pub fn cancellation_expired(&self, transaction: &Transaction) {
        let topics = (
            symbol_short!("can_exp"),
            transaction.id,
            transaction.buyer.clone(),
            transaction.seller.clone(),
        );
        self.env.events().publish(
            topics, 
            transaction.status
        );
    }
} 