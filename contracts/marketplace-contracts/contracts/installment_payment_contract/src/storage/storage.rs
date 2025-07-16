use soroban_sdk::{symbol_short, Env, Symbol};

use crate::storage::contracts::InstallmentAgreement;

// const ADMIN: Symbol = symbol_short!("i_p_admin"); // length cannot be more than 9, hence, i = installment, p = payment,
const AGREEMENT_ID: Symbol = symbol_short!("agree_id");
const AGREEMENT: Symbol = symbol_short!("agreement");

pub fn get_installment_agreement(env: &Env, agreement_id: u128) -> Option<InstallmentAgreement> {
    let agreement_key: (u128, Symbol) = (agreement_id, AGREEMENT);

    env.storage().persistent().get(&agreement_key)
}

pub fn save_installment_agreement(
    env: &Env,
    agreement_id: u128,
    installment_agreement: InstallmentAgreement,
) {
    let agreement_key: (u128, Symbol) = (agreement_id, AGREEMENT);
    env.storage()
        .persistent()
        .set(&agreement_key, &installment_agreement);
}

pub fn get_agreement_id(env: &Env) -> u128 {
    env.storage().persistent().get(&AGREEMENT_ID).unwrap_or(0)
}

pub fn save_new_agreement_id(env: &Env, new_agreement_id: u128) {
    env.storage()
        .persistent()
        .set(&AGREEMENT_ID, &new_agreement_id);
}
