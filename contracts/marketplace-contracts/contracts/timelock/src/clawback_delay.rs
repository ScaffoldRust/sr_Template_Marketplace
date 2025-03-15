use soroban_sdk::Env;

use crate::storage_types::DataKey;

pub fn read_clawback_delay(env: &Env) -> u64 {
    let key = DataKey::ClawbackDelay;
    env.storage().instance().get(&key).unwrap()
}

pub fn write_clawback_delay(env: &Env, clawback_delay: u64) {
    let key = DataKey::ClawbackDelay;
    env.storage().instance().set(&key, &clawback_delay);
}
