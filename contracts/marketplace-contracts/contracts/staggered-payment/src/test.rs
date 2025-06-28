#[cfg(test)]
mod test_ {
    use crate::staggered_payments::{StaggeredPaymentContract, StaggeredPaymentContractClient};
    use crate::storage_types::{DataKey, Transactions, TIMEOUT};
    use soroban_sdk::log;
    use soroban_sdk::testutils::{Address as _, Events, Ledger};
    use soroban_sdk::{symbol_short, vec};
    use soroban_sdk::{token, Address, Env, IntoVal, Symbol};
    use soroban_sdk::{FromVal, Val, Vec};

    // Helper to create a token contract for testing
    fn create_token_contract<'a>(
        e: &Env,
        admin: &Address,
    ) -> (Address, token::StellarAssetClient<'a>) {
        let contract_address = e.register_stellar_asset_contract_v2(admin.clone());
        (
            contract_address.address(),
            token::StellarAssetClient::new(e, &contract_address.address()),
        )
    }

    // Helper to create test users with initial token balance
    fn create_user(
        env: &Env,
        token: &token::StellarAssetClient,
        admin: &Address,
        amount: i128,
    ) -> Address {
        let user = Address::generate(env);

        token.mint(&user, &amount);
        token.mint(admin, &amount);
        user
    }

    fn setup_env() -> (
        Env,
        StaggeredPaymentContractClient<'static>,
        Address,
        Address,
        Address,
    ) {
        let env = Env::default();

        env.mock_all_auths();
        let admin = Address::generate(&env);

        let (token_address, token_client) = create_token_contract(&env, &admin);
        let contract_id = env.register(StaggeredPaymentContract, ());

        let buyer = create_user(&env, &token_client, &admin, 10000);
        let contract = StaggeredPaymentContractClient::new(&env, &contract_id);
        contract.initialize(&token_address);
        let seller = soroban_sdk::Address::generate(&env);
        let token = token_address;
        (env, contract, buyer, seller, token)
    }

    fn get_contract_events(
        env: &Env,
        _client: &StaggeredPaymentContractClient,
    ) -> Vec<(soroban_sdk::Address, Vec<Val>, Val)> {
        let events = env.events().all();
        log!(env, "events inside get_contract_events: {:?}", events);
        events
    }

    #[test]
    fn test_event_emission() {
        let (env, client, buyer, seller, _) = setup_env();

        env.mock_all_auths();

        let tx_id = client.create_transaction(
            &buyer,
            &seller,
            &1000,
            &vec![&env, 50, 50],
            &vec![&env, symbol_short!("design"), symbol_short!("develop")],
        );

        let events = get_contract_events(&env, &client);
        log!(&env, "Events in test_event_emission: {:?}", events);

        assert!(tx_id >= 1, "Transaction ID should be 1");

        let new_tx_event = events.iter().find(|event| {
            let contract_address_val: Val = client.address.clone().into_val(&env);
            event.0 == Address::from_val(&env, &contract_address_val)
                && !event.1.is_empty()
                && Symbol::from_val(&env, &event.1.get(0).unwrap()) == symbol_short!("new_tx")
        });

        assert!(
            new_tx_event.is_some(),
            "Expected a 'new_tx' event to be emitted."
        );

        let event = new_tx_event.unwrap();
        let topics = &event.1;
        let topic0: Symbol = Symbol::from_val(&env, &topics.get(0).unwrap());
        let topic1: u32 = u32::from_val(&env, &topics.get(1).unwrap());

        assert_eq!(
            topic0,
            symbol_short!("new_tx"),
            "Topic 0 should be 'new_tx'"
        );
        assert_eq!(topic1, tx_id, "Topic 1 should be the transaction ID");

        let data = &event.2;
        let (event_buyer, event_seller, event_total_amount): (Address, Address, i128) =
            FromVal::from_val(&env, data);
        assert_eq!(event_buyer, buyer);
        assert_eq!(event_seller, seller);
        assert_eq!(event_total_amount, 1000);
    }

    #[test]
    fn test_create_transaction() {
        let (env, client, buyer, seller, _) = setup_env();
        let total_amount = 1000;
        let milestone_percentages = vec![&env, 50, 50];
        let milestone_descriptions = vec![&env, symbol_short!("design"), symbol_short!("develop")];
        env.mock_all_auths();

        let tx_id = client.create_transaction(
            &buyer,
            &seller,
            &total_amount,
            &milestone_percentages,
            &milestone_descriptions,
        );

        let events = get_contract_events(&env, &client);
        log!(&env, "Events in test_create_transaction: {:?}", events);

        assert_eq!(tx_id, 1, "Transaction ID should be 1");

        let transaction: Transactions = env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&DataKey::Transaction(tx_id))
                .unwrap()
        });
        assert_eq!(transaction.buyer, buyer);
        assert_eq!(transaction.seller, seller);
        assert_eq!(transaction.total_amount, total_amount);
        assert_eq!(transaction.milestones.len(), 2);
        assert_eq!(transaction.milestones.get_unchecked(0).amount, 500);
        assert_eq!(transaction.milestones.get_unchecked(1).amount, 500);
        let new_tx_event = events.iter().find(|event| {
            let contract_address_val: Val = client.address.clone().into_val(&env);
            event.0 == Address::from_val(&env, &contract_address_val)
                && !event.1.is_empty()
                && Symbol::from_val(&env, &event.1.get(0).unwrap()) == symbol_short!("new_tx")
        });

        assert!(
            new_tx_event.is_some(),
            "Expected a 'new_tx' event to be emitted."
        );

        let event = new_tx_event.unwrap();
        let topics = &event.1;
        let topic0: Symbol = Symbol::from_val(&env, &topics.get(0).unwrap());
        let topic1: u32 = u32::from_val(&env, &topics.get(1).unwrap());

        assert_eq!(
            topic0,
            symbol_short!("new_tx"),
            "Topic 0 should be 'new_tx'"
        );
        assert_eq!(topic1, tx_id, "Topic 1 should be the transaction ID");
    }

    #[test]
    fn test_milestone_workflow() {
        let (env, client, buyer, seller, _) = setup_env();
        let total_amount = 1000;
        let milestone_percentages = vec![&env, 50, 50];
        let milestone_descriptions = vec![&env, symbol_short!("design"), symbol_short!("develop")];

        env.mock_all_auths();

        let tx_id = client.create_transaction(
            &buyer,
            &seller,
            &total_amount,
            &milestone_percentages,
            &milestone_descriptions,
        );

        client.submit_milestone(&tx_id, &0);

        client.approve_milestone(&tx_id, &0);

        let events = get_contract_events(&env, &client);
        log!(&env, "Events in test_milestone_workflow: {:?}", events);

        let transaction: Transactions = env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&DataKey::Transaction(tx_id))
                .unwrap()
        });
        assert!(transaction.milestones.get_unchecked(0).completed);
        assert!(transaction.milestones.get_unchecked(0).approved);

        assert!(!events.is_empty(), "Expected two events (submit, approve)");
        if events.len() >= 2 {
            let event = events.get(1).unwrap();
            let topics = &event.1;
            let topic0_val = topics.get(0).unwrap();
            let topic0: Symbol = Symbol::from_val(&env, &topic0_val);
            assert_eq!(topic0, symbol_short!("approve"));
        }
    }

    #[test]
    fn test_timeout() {
        let (env, client, buyer, seller, _) = setup_env();
        let total_amount = 1000;
        let milestone_percentages = vec![&env, 50, 50];
        let milestone_descriptions = vec![&env, symbol_short!("design"), symbol_short!("develop")];

        env.mock_all_auths();

        let tx_id = client.create_transaction(
            &buyer,
            &seller,
            &total_amount,
            &milestone_percentages,
            &milestone_descriptions,
        );

        client.submit_milestone(&tx_id, &0);

        env.ledger().with_mut(|l| l.timestamp += TIMEOUT + 1);

        client.check_timeout(&tx_id, &0);

        let events = get_contract_events(&env, &client);
        log!(&env, "Events in test_timeout: {:?}", events);

        let transaction: Transactions = env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&DataKey::Transaction(tx_id))
                .unwrap()
        });
        assert!(transaction.milestones.get_unchecked(0).completed);
        assert!(transaction.milestones.get_unchecked(0).approved);

        assert!(!events.is_empty(), "Expected two events (submit, timeout)");
        if events.len() >= 2 {
            let event = events.get(1).unwrap();
            let topics = &event.1;
            let topic0_val = topics.get(0).unwrap();
            let topic0: Symbol = Symbol::from_val(&env, &topic0_val);
            assert_eq!(topic0, symbol_short!("timeout"));
        }
    }

    #[test]
    fn test_dispute() {
        let (env, client, buyer, seller, arbiter) = setup_env();
        let total_amount = 1000;
        let milestone_percentages = vec![&env, 50, 50];
        let milestone_descriptions = vec![&env, symbol_short!("design"), symbol_short!("develop")];

        env.mock_all_auths();

        let tx_id = client.create_transaction(
            &buyer,
            &seller,
            &total_amount,
            &milestone_percentages,
            &milestone_descriptions,
        );

        client.submit_milestone(&tx_id, &0);

        client.dispute_milestone(&tx_id, &0);

        client.resolve_dispute(&tx_id, &0, &false, &arbiter);

        let events = get_contract_events(&env, &client);
        log!(&env, "Events in test_dispute: {:?}", events);

        let transaction: Transactions = env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get(&DataKey::Transaction(tx_id))
                .unwrap()
        });
        assert!(!transaction.milestones.get_unchecked(0).disputed);
        assert!(!transaction.milestones.get_unchecked(0).approved);

        assert!(
            !events.is_empty(),
            "Expected three events (submit, dispute, refund)"
        );
        if events.len() >= 3 {
            let event = events.get(2).unwrap();
            let topics = &event.1;
            let topic0_val = topics.get(0).unwrap();
            let topic0: Symbol = Symbol::from_val(&env, &topic0_val);
            assert_eq!(topic0, symbol_short!("refund"));

            let event = events.get(3).unwrap();
            let topics = &event.1;
            let topic0_val = topics.get(0).unwrap();
            let topic0: Symbol = Symbol::from_val(&env, &topic0_val);
            assert_eq!(topic0, symbol_short!("tx_done"));
        }
    }

    #[test]
    #[should_panic(expected = "Milestone not found")]
    fn test_invalid_milestone() {
        let (env, client, buyer, seller, _) = setup_env();
        let total_amount = 1000;
        let milestone_percentages = vec![&env, 50, 50];
        let milestone_descriptions = vec![&env, symbol_short!("design"), symbol_short!("develop")];

        env.mock_all_auths();

        let tx_id = client.create_transaction(
            &buyer,
            &seller,
            &total_amount,
            &milestone_percentages,
            &milestone_descriptions,
        );

        client.submit_milestone(&tx_id, &999);
    }
}
