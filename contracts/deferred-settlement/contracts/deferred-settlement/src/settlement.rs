use crate::admin::is_admin;
use crate::types::{
    DataKey, DeferredTransaction, Error, SettlementCondition, TransactionEvent, TransactionStatus,
};
use soroban_sdk::{token, Address, Env, Symbol};

pub fn verify_condition(
    env: Env,
    caller: Address,
    transaction_id: u128,
    oracle_input: Option<bool>,
) -> Result<(), Error> {
    caller.require_auth();
    let mut transaction: DeferredTransaction = env
        .storage()
        .instance()
        .get(&DataKey::Transaction(transaction_id))
        .ok_or(Error::TransactionNotFound)?;

    if transaction.status != TransactionStatus::Pending {
        return Err(Error::NotPending);
    }

    let is_verified = match transaction.condition {
        SettlementCondition::TimeBased => env.ledger().timestamp() >= transaction.deadline,
        SettlementCondition::BuyerApproval => caller == transaction.buyer,
        SettlementCondition::OracleConfirmation => {
            if !is_admin(&env, &caller) {
                return Err(Error::Unauthorized);
            }
            oracle_input.unwrap_or(false)
        }
    };

    if !is_verified {
        return Err(Error::ConditionNotMet);
    }

    // Update status
    transaction.status = TransactionStatus::Completed;
    env.storage()
        .instance()
        .set(&DataKey::Transaction(transaction_id), &transaction);

    // Release funds
    let token_contract_id = env
        .storage()
        .instance()
        .get(&DataKey::TokenContract)
        .ok_or(Error::TokenContractNotSet)?;
    let token_client = token::Client::new(&env, &token_contract_id);
    token_client.transfer(
        &env.current_contract_address(),
        &transaction.seller,
        &transaction.amount,
    );

    // Emit events
    env.events().publish(
        (
            "DeferredSettlement",
            Symbol::new(&env, "condition_verified"),
        ),
        TransactionEvent::ConditionVerified(transaction_id, Symbol::new(&env, "verified")),
    );
    env.events().publish(
        ("DeferredSettlement", Symbol::new(&env, "funds_released")),
        TransactionEvent::FundsReleased(transaction_id, transaction.seller),
    );

    Ok(())
}
