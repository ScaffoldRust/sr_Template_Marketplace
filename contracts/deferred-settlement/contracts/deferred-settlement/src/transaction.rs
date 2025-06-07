use crate::types::{
    DataKey, DeferredTransaction, Error, SettlementCondition, TransactionEvent, TransactionStatus,
};
use soroban_sdk::{token, Address, Env, Symbol};

pub fn create_transaction(
    env: Env,
    buyer: Address,
    seller: Address,
    amount: i128,
    condition: SettlementCondition,
    duration: u64,
) -> Result<u128, Error> {
    buyer.require_auth();

    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    let transaction_id: u128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalTransactions)
        .unwrap_or(0);

    let transaction = DeferredTransaction {
        id: transaction_id,
        buyer: buyer.clone(),
        seller: seller.clone(),
        amount,
        condition,
        created_at: env.ledger().timestamp(),
        deadline: env.ledger().timestamp() + duration,
        status: TransactionStatus::Pending,
    };

    // Get the token contract ID from storage or environment
    let token_contract_id = env
        .storage()
        .instance()
        .get(&DataKey::TokenContract)
        .ok_or(Error::TokenContractNotSet)?;
    let token_client = token::Client::new(&env, &token_contract_id);
    token_client.transfer(&buyer, &env.current_contract_address(), &amount);

    // Store transaction
    env.storage()
        .instance()
        .set(&DataKey::Transaction(transaction_id), &transaction);
    env.storage()
        .instance()
        .set(&DataKey::TotalTransactions, &(transaction_id + 1));

    // Emit event
    env.events().publish(
        (
            "DeferredSettlement",
            Symbol::new(&env, "transaction_created"),
        ),
        TransactionEvent::TransactionCreated(transaction_id, buyer, seller, amount),
    );

    Ok(transaction_id)
}
