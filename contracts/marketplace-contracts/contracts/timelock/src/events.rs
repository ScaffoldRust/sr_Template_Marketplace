use crate::deposit::Deposit;
use soroban_sdk::{symbol_short, Env, Symbol};

pub struct Events {
    env: Env,
}

impl Events {
    #[inline(always)]
    pub fn new(env: &Env) -> Events {
        Events { env: env.clone() }
    }

    pub fn deposit(&self, deposit: Deposit) {
        let topics = (
            symbol_short!("deposit"),
            deposit.depositor,
            deposit.withdrawer,
            deposit.token,
        );
        self.env
            .events()
            .publish(topics, (deposit.amount, deposit.unlock_timestamp));
    }

    pub fn withdrawal(&self, deposit: Deposit, success: bool) {
        let topics = (
            Symbol::new(&self.env, "withdrawal"),
            deposit.depositor,
            deposit.withdrawer,
            deposit.token,
            success,
        );
        self.env
            .events()
            .publish(topics, (deposit.amount, deposit.unlock_timestamp));
    }

    pub fn clawback(&self, deposit: Deposit, success: bool) {
        let topics = (
            symbol_short!("clawback"),
            deposit.depositor,
            deposit.withdrawer,
            deposit.token,
            success,
        );
        self.env
            .events()
            .publish(topics, (deposit.amount, deposit.unlock_timestamp));
    }
}
