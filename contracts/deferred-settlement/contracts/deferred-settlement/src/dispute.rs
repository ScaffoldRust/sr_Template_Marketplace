use crate::types::{DataKey, DeferredTransaction, Error, TransactionEvent, TransactionStatus};
use soroban_sdk::{token, Address, Env, Symbol};

pub fn initiate_dispute(env: Env, caller: Address, transaction_id: u128) -> Result<(), Error> {
    caller.require_auth();

    let mut transaction: DeferredTransaction = env
        .storage()
        .instance()
        .get(&DataKey::Transaction(transaction_id))
        .ok_or(Error::TransactionNotFound)?;

    if transaction.status != TransactionStatus::Pending {
        return Err(Error::NotPending);
    }

    if caller != transaction.buyer && caller != transaction.seller {
        return Err(Error::Unauthorized);
    }

    transaction.status = TransactionStatus::Disputed;
    env.storage()
        .instance()
        .set(&DataKey::Transaction(transaction_id), &transaction);

    env.events().publish(
        ("DeferredSettlement", Symbol::new(&env, "dispute_initiated")),
        TransactionEvent::DisputeInitiated(transaction_id),
    );

    Ok(())
}

pub fn resolve_dispute(
    env: Env,
    caller: Address,
    transaction_id: u128,
    release_to_seller: bool,
) -> Result<(), Error> {
    caller.require_auth();

    let mut transaction: DeferredTransaction = env
        .storage()
        .instance()
        .get(&DataKey::Transaction(transaction_id))
        .ok_or(Error::TransactionNotFound)?;

    if transaction.status != TransactionStatus::Disputed {
        return Err(Error::NotPending);
    }

    if caller != transaction.buyer && caller != transaction.seller {
        return Err(Error::Unauthorized);
    }

    // Get the token contract ID from storage or environment
    let token_contract_id = env
        .storage()
        .instance()
        .get(&DataKey::TokenContract)
        .ok_or(Error::TokenContractNotSet)?;
    let token_client = token::Client::new(&env, &token_contract_id);

    if release_to_seller {
        token_client.transfer(
            &env.current_contract_address(),
            &transaction.seller,
            &transaction.amount,
        );
        transaction.status = TransactionStatus::Completed;
    } else {
        token_client.transfer(
            &env.current_contract_address(),
            &transaction.buyer,
            &transaction.amount,
        );
        transaction.status = TransactionStatus::Refunded;
    }

    env.storage()
        .instance()
        .set(&DataKey::Transaction(transaction_id), &transaction);

    env.events().publish(
        ("DeferredSettlement", Symbol::new(&env, "dispute_resolved")),
        TransactionEvent::DisputeResolved(transaction_id, release_to_seller),
    );

    Ok(())
}
