use soroban_sdk::Env;

use crate::storage_types::DataKey;

pub fn read_lock_duration(env: &Env) -> u64 {
    let key = DataKey::LockDuration;
    env.storage().instance().get(&key).unwrap()
}

pub fn write_lock_duration(env: &Env, lock_duration: u64) {
    let key = DataKey::LockDuration;
    env.storage().instance().set(&key, &lock_duration);
}
