use crate::clawback_delay::{read_clawback_delay, write_clawback_delay};
use crate::deposit::{read_deposit, remove_deposit, write_deposit, Deposit};
use crate::events::Events;
use crate::lock_duration::{read_lock_duration, write_lock_duration};
use crate::storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use soroban_sdk::{contract, contractimpl, token, Address, Env};

#[contract]
pub struct Timelock;

#[contractimpl]
impl Timelock {
    pub fn __constructor(env: Env, lock_duration: u64, clawback_delay: u64) {
        write_lock_duration(&env, lock_duration);
        write_clawback_delay(&env, clawback_delay);
    }

    pub fn lock_duration(env: Env) -> u64 {
        read_lock_duration(&env)
    }

    pub fn clawback_delay(env: Env) -> u64 {
        read_clawback_delay(&env)
    }

    pub fn deposit(
        env: Env,
        depositor: Address,
        withdrawer: Address,
        token: Address,
        amount: i128,
    ) {
        // Make sure `depositor` address authorized the deposit call with all the arguments.
        depositor.require_auth();
        if let Some(deposit) = read_deposit(&env, &depositor, &withdrawer, &token) {
            panic!(
                "TimelockDuplicateDeposit({:?}, {:?}, {:?})",
                deposit.depositor, deposit.withdrawer, deposit.token
            );
        }
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        // Transfer token from `depositor` to this contract address.
        token::Client::new(&env, &token).transfer(
            &depositor,
            &env.current_contract_address(),
            &amount,
        );
        // Store all the necessary info to allow the withdrawer to claim it.
        let deposit = Deposit {
            depositor,
            withdrawer,
            token,
            amount,
            unlock_timestamp: env.ledger().timestamp() + read_lock_duration(&env),
        };
        write_deposit(&env, &deposit);
        Events::new(&env).deposit(deposit);
    }

    pub fn get_deposit(
        env: Env,
        depositor: Address,
        withdrawer: Address,
        token: Address,
    ) -> Option<Deposit> {
        read_deposit(&env, &depositor, &withdrawer, &token)
    }

    pub fn withdraw(env: Env, depositor: Address, withdrawer: Address, token: Address) {
        // Make sure withdrawer has authorized this call, which ensures their identity.
        withdrawer.require_auth();
        let deposit_opt = read_deposit(&env, &depositor, &withdrawer, &token);
        if deposit_opt.is_none() {
            panic!(
                "TimelockNonexistentDeposit({:?}, {:?}, {:?})",
                depositor, withdrawer, token
            );
        }
        let deposit = deposit_opt.unwrap();
        if env.ledger().timestamp() < deposit.unlock_timestamp {
            Events::new(&env).withdrawal(deposit, false);
            panic!(
                "TimelockLockedDeposit({:?}, {:?}, {:?})",
                depositor, withdrawer, token
            );
        }
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        // Transfer the stored amount of token to withdrawer after passing all the checks.
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &withdrawer,
            &deposit.amount,
        );
        // Remove the deposit entry to prevent any further withdrawals.
        remove_deposit(&env, &deposit);
        Events::new(&env).withdrawal(deposit, true);
    }

    pub fn clawback(env: Env, depositor: Address, withdrawer: Address, token: Address) {
        // Make sure `depositor` address authorized the clawback call with all the arguments.
        depositor.require_auth();
        let deposit_opt = read_deposit(&env, &depositor, &withdrawer, &token);
        if deposit_opt.is_none() {
            panic!(
                "TimelockNonexistentDeposit({:?}, {:?}, {:?})",
                depositor, withdrawer, token
            );
        }
        let deposit = deposit_opt.unwrap();
        if env.ledger().timestamp() < deposit.unlock_timestamp + read_clawback_delay(&env) {
            Events::new(&env).clawback(deposit, false);
            panic!(
                "TimelockLockedClawback({:?}, {:?}, {:?})",
                depositor, withdrawer, token
            );
        }
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        // Transfer back the stored amount of token to depositor after passing all the checks.
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &depositor,
            &deposit.amount,
        );
        // Remove the deposit entry to prevent any further clawbacks.
        remove_deposit(&env, &deposit);
        Events::new(&env).clawback(deposit, true);
    }
}
