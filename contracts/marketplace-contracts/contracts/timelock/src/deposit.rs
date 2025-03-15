use crate::storage_types::{DataKey, DEPOSIT_BUMP_AMOUNT, DEPOSIT_LIFETIME_THRESHOLD};
use soroban_sdk::{contracttype, Address, Env};

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub struct Deposit {
    pub depositor: Address,
    pub withdrawer: Address,
    pub token: Address,
    pub amount: i128,
    pub unlock_timestamp: u64,
}

pub fn read_deposit(
    env: &Env,
    depositor: &Address,
    withdrawer: &Address,
    token: &Address,
) -> Option<Deposit> {
    let key = DataKey::Deposit(depositor.clone(), withdrawer.clone(), token.clone());
    if let Some(deposit) = env.storage().persistent().get::<DataKey, Deposit>(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            DEPOSIT_LIFETIME_THRESHOLD,
            DEPOSIT_BUMP_AMOUNT,
        );
        Some(deposit)
    } else {
        None
    }
}

pub fn write_deposit(env: &Env, deposit: &Deposit) {
    let Deposit {
        depositor,
        withdrawer,
        token,
        ..
    } = deposit.clone();
    let key = DataKey::Deposit(depositor, withdrawer, token);
    env.storage().persistent().set(&key, deposit);
    env.storage()
        .persistent()
        .extend_ttl(&key, DEPOSIT_LIFETIME_THRESHOLD, DEPOSIT_BUMP_AMOUNT);
}

pub fn remove_deposit(env: &Env, deposit: &Deposit) {
    let Deposit {
        depositor,
        withdrawer,
        token,
        ..
    } = deposit.clone();
    let key = DataKey::Deposit(depositor, withdrawer, token);
    env.storage().persistent().remove(&key);
}
