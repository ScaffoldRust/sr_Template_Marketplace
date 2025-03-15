#![cfg(test)]
extern crate std;

use crate::deposit::Deposit;
use crate::{contract::Timelock, TimelockClient};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events, Ledger},
    token, vec, Address, Env, IntoVal, Symbol,
};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

const LOCK_DURATION: u64 = 24 * 60 * 60;
const CLAWBACK_DELAY: u64 = LOCK_DURATION * 7;
const DEPOSIT_AMOUNT: i128 = 1000;

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(env, &sac.address()),
        token::StellarAssetClient::new(env, &sac.address()),
    )
}

fn create_timelock_contract<'a>(env: &Env) -> TimelockClient<'a> {
    let timelock_contract_address = env.register(Timelock, (LOCK_DURATION, CLAWBACK_DELAY));
    TimelockClient::new(env, &timelock_contract_address)
}

struct TimelockTest<'a> {
    env: Env,
    depositor: Address,
    withdrawer: Address,
    token: TokenClient<'a>,
    timelock: TimelockClient<'a>,
}

impl<'a> TimelockTest<'a> {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        let depositor = Address::generate(&env);
        let withdrawer = Address::generate(&env);
        let token_admin = Address::generate(&env);
        let (token, token_admin_client) = create_token_contract(&env, &token_admin);
        token_admin_client.mint(&depositor, &DEPOSIT_AMOUNT);
        let timelock = create_timelock_contract(&env);
        TimelockTest {
            env,
            depositor,
            withdrawer,
            token,
            timelock,
        }
    }
    fn with_deposit(&self) -> &Self {
        self.timelock.deposit(
            &self.depositor,
            &self.withdrawer,
            &self.token.address,
            &DEPOSIT_AMOUNT,
        );
        self
    }
    fn setup_with_deposit() -> Self {
        let test = Self::setup();
        test.with_deposit();
        test
    }
}

#[test]
fn test_lock_duration() {
    let TimelockTest { timelock, .. } = TimelockTest::setup();
    assert_eq!(timelock.lock_duration(), LOCK_DURATION);
}

#[test]
fn test_clawback_delay() {
    let TimelockTest { timelock, .. } = TimelockTest::setup();
    assert_eq!(timelock.clawback_delay(), CLAWBACK_DELAY);
}

#[test]
fn test_deposit_authorization() {
    let TimelockTest {
        env,
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup();
    timelock.deposit(&depositor, &withdrawer, &token.address, &DEPOSIT_AMOUNT);
    assert_eq!(
        env.auths(),
        [(
            depositor.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    timelock.address.clone(),
                    symbol_short!("deposit"),
                    (
                        depositor.clone(),
                        withdrawer,
                        token.address.clone(),
                        DEPOSIT_AMOUNT
                    )
                        .into_val(&env)
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token.address,
                        symbol_short!("transfer"),
                        (depositor, timelock.address, DEPOSIT_AMOUNT).into_val(&env)
                    )),
                    sub_invocations: std::vec![]
                }]
            }
        )]
    );
}

#[test]
#[should_panic(
    expected = "TimelockDuplicateDeposit(Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM), Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFCT4), Contract(CCABDO7UZXYE4W6GVSEGSNNZTKSLFQGKXXQTH6OX7M7GKZ4Z6CUJNGZN))"
)]
fn test_deposit_duplicate() {
    let TimelockTest {
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup();
    timelock.deposit(&depositor, &withdrawer, &token.address, &DEPOSIT_AMOUNT);
    timelock.deposit(&depositor, &withdrawer, &token.address, &DEPOSIT_AMOUNT);
}

#[test]
#[should_panic(expected = "balance is not sufficient to spend")]
fn test_deposit_insufficient_balance() {
    let TimelockTest {
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup();
    timelock.deposit(
        &depositor,
        &withdrawer,
        &token.address,
        &(DEPOSIT_AMOUNT * 2),
    );
}

#[test]
fn test_deposit_success() {
    let TimelockTest {
        env,
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup();
    let depositor_balance_before = token.balance(&depositor);
    let timelock_balance_before = token.balance(&timelock.address);
    timelock.deposit(&depositor, &withdrawer, &token.address, &DEPOSIT_AMOUNT);
    assert_eq!(
        token.balance(&depositor),
        depositor_balance_before - DEPOSIT_AMOUNT
    );
    assert_eq!(
        token.balance(&timelock.address),
        timelock_balance_before + DEPOSIT_AMOUNT
    );
    assert_eq!(
        timelock.get_deposit(&depositor, &withdrawer, &token.address),
        Some(Deposit {
            depositor,
            withdrawer,
            token: token.address,
            amount: DEPOSIT_AMOUNT,
            unlock_timestamp: env.ledger().timestamp() + LOCK_DURATION
        })
    );
}

#[test]
fn test_deposit_events() {
    let TimelockTest {
        env,
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup();
    timelock.deposit(&depositor, &withdrawer, &token.address, &DEPOSIT_AMOUNT);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                token.address.clone(),
                (
                    symbol_short!("transfer"),
                    depositor.clone(),
                    timelock.address.clone(),
                    token.name()
                )
                    .into_val(&env),
                (DEPOSIT_AMOUNT).into_val(&env)
            ),
            (
                timelock.address,
                (
                    symbol_short!("deposit"),
                    depositor,
                    withdrawer,
                    token.address
                )
                    .into_val(&env),
                (DEPOSIT_AMOUNT, env.ledger().timestamp() + LOCK_DURATION).into_val(&env)
            )
        ]
    );
}

#[test]
fn test_withdraw_authorization() {
    let TimelockTest {
        env,
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup_with_deposit();
    env.ledger().with_mut(|li| li.timestamp = LOCK_DURATION);
    timelock.withdraw(&depositor, &withdrawer, &token.address);
    assert_eq!(
        env.auths(),
        [(
            withdrawer.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    timelock.address,
                    symbol_short!("withdraw"),
                    (depositor, withdrawer, token.address).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

#[test]
#[should_panic(
    expected = "TimelockNonexistentDeposit(Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM), Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM), Contract(CCABDO7UZXYE4W6GVSEGSNNZTKSLFQGKXXQTH6OX7M7GKZ4Z6CUJNGZN))"
)]
fn test_withdraw_non_existent_deposit() {
    let TimelockTest {
        depositor,
        token,
        timelock,
        ..
    } = TimelockTest::setup_with_deposit();
    timelock.withdraw(&depositor, &depositor, &token.address);
}

#[test]
#[should_panic(
    expected = "TimelockLockedDeposit(Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM), Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFCT4), Contract(CCABDO7UZXYE4W6GVSEGSNNZTKSLFQGKXXQTH6OX7M7GKZ4Z6CUJNGZN))"
)]
fn test_withdraw_locked_deposit() {
    let TimelockTest {
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup_with_deposit();
    timelock.withdraw(&depositor, &withdrawer, &token.address);
}

#[test]
fn test_withdraw_success() {
    let TimelockTest {
        env,
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup_with_deposit();
    env.ledger().with_mut(|li| li.timestamp = LOCK_DURATION);
    let timelock_balance_before = token.balance(&timelock.address);
    let withdrawer_balance_before = token.balance(&withdrawer);
    timelock.withdraw(&depositor, &withdrawer, &token.address);
    assert_eq!(
        token.balance(&timelock.address),
        timelock_balance_before - DEPOSIT_AMOUNT
    );
    assert_eq!(
        token.balance(&withdrawer),
        withdrawer_balance_before + DEPOSIT_AMOUNT
    );
    assert_eq!(
        timelock.get_deposit(&depositor, &withdrawer, &token.address),
        None
    );
}

#[test]
fn test_withdraw_events() {
    let TimelockTest {
        env,
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup_with_deposit();
    let _ = timelock.try_withdraw(&depositor, &withdrawer, &token.address);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                timelock.address.clone(),
                (
                    Symbol::new(&env, "withdrawal"),
                    depositor.clone(),
                    withdrawer.clone(),
                    token.address.clone(),
                    false
                )
                    .into_val(&env),
                (DEPOSIT_AMOUNT, LOCK_DURATION).into_val(&env)
            )
        ]
    );
    env.ledger().with_mut(|li| li.timestamp = LOCK_DURATION);
    timelock.withdraw(&depositor, &withdrawer, &token.address);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                token.address.clone(),
                (
                    symbol_short!("transfer"),
                    timelock.address.clone(),
                    withdrawer.clone(),
                    token.name()
                )
                    .into_val(&env),
                (DEPOSIT_AMOUNT).into_val(&env)
            ),
            (
                timelock.address,
                (
                    Symbol::new(&env, "withdrawal"),
                    depositor,
                    withdrawer,
                    token.address,
                    true
                )
                    .into_val(&env),
                (DEPOSIT_AMOUNT, LOCK_DURATION).into_val(&env)
            )
        ]
    );
}

#[test]
fn test_clawback_authorization() {
    let TimelockTest {
        env,
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup_with_deposit();
    env.ledger()
        .with_mut(|li| li.timestamp = LOCK_DURATION + CLAWBACK_DELAY);
    timelock.clawback(&depositor, &withdrawer, &token.address);
    assert_eq!(
        env.auths(),
        [(
            depositor.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    timelock.address,
                    symbol_short!("clawback"),
                    (depositor, withdrawer, token.address).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

#[test]
#[should_panic(
    expected = "TimelockNonexistentDeposit(Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM), Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFCT4), Contract(CCABDO7UZXYE4W6GVSEGSNNZTKSLFQGKXXQTH6OX7M7GKZ4Z6CUJNGZN))"
)]
fn test_clawback_non_existent_deposit() {
    let TimelockTest {
        env,
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup_with_deposit();
    env.ledger().with_mut(|li| li.timestamp = LOCK_DURATION);
    timelock.withdraw(&depositor, &withdrawer, &token.address);
    env.ledger()
        .with_mut(|li| li.timestamp = LOCK_DURATION + CLAWBACK_DELAY);
    timelock.clawback(&depositor, &withdrawer, &token.address);
}

#[test]
#[should_panic(
    expected = "TimelockLockedClawback(Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM), Contract(CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFCT4), Contract(CCABDO7UZXYE4W6GVSEGSNNZTKSLFQGKXXQTH6OX7M7GKZ4Z6CUJNGZN))"
)]
fn test_clawback_locked_clawback() {
    let TimelockTest {
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup_with_deposit();
    timelock.clawback(&depositor, &withdrawer, &token.address);
}

#[test]
fn test_clawback_success() {
    let TimelockTest {
        env,
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup_with_deposit();
    env.ledger()
        .with_mut(|li| li.timestamp = LOCK_DURATION + CLAWBACK_DELAY);
    let timelock_balance_before = token.balance(&timelock.address);
    let depositor_balance_before = token.balance(&depositor);
    timelock.clawback(&depositor, &withdrawer, &token.address);
    assert_eq!(
        token.balance(&timelock.address),
        timelock_balance_before - DEPOSIT_AMOUNT
    );
    assert_eq!(
        token.balance(&depositor),
        depositor_balance_before + DEPOSIT_AMOUNT
    );
    assert_eq!(
        timelock.get_deposit(&depositor, &withdrawer, &token.address),
        None
    );
}

#[test]
fn test_clawback_events() {
    let TimelockTest {
        env,
        depositor,
        withdrawer,
        token,
        timelock,
        ..
    } = TimelockTest::setup_with_deposit();
    let _ = timelock.try_clawback(&depositor, &withdrawer, &token.address);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                timelock.address.clone(),
                (
                    Symbol::new(&env, "clawback"),
                    depositor.clone(),
                    withdrawer.clone(),
                    token.address.clone(),
                    false
                )
                    .into_val(&env),
                (DEPOSIT_AMOUNT, LOCK_DURATION).into_val(&env)
            )
        ]
    );
    env.ledger()
        .with_mut(|li| li.timestamp = LOCK_DURATION + CLAWBACK_DELAY);
    timelock.clawback(&depositor, &withdrawer, &token.address);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                token.address.clone(),
                (
                    symbol_short!("transfer"),
                    timelock.address.clone(),
                    depositor.clone(),
                    token.name()
                )
                    .into_val(&env),
                (DEPOSIT_AMOUNT).into_val(&env)
            ),
            (
                timelock.address,
                (
                    Symbol::new(&env, "clawback"),
                    depositor,
                    withdrawer,
                    token.address,
                    true
                )
                    .into_val(&env),
                (DEPOSIT_AMOUNT, LOCK_DURATION).into_val(&env)
            )
        ]
    );
}
