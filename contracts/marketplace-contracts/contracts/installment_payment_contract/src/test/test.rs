#[cfg(test)]
mod test {

    use crate::installment_payment::installment::{InstallmentPayment, InstallmentPaymentClient};
    use soroban_sdk::{
        log,
        testutils::{Address as _, Ledger},
        token::{self, StellarAssetClient},
        Address, Env, String,
    };

    fn create_contract_variables() -> (Env, Address, Address) {
        let env: Env = Env::default();
        env.mock_all_auths();

        let contract_address: Address = env.register(InstallmentPayment, {});
        let mocked_address: Address = Address::generate(&env);

        return (env, contract_address, mocked_address);
    }
    #[test]
    fn test_initialized_contract() {
        // register the contract

        let (env, contract_address, admin) = create_contract_variables();
        // env.register_contract(None, InstallmentPayment); the register contract method of the env has been deprecated

        let installment_payment_instance = InstallmentPaymentClient::new(&env, &contract_address); // the client with the new function return the instance of the contract
                                                                                                   // installment_payment_instance
        let initialized_result = installment_payment_instance.initialize(&admin);

        assert_eq!(initialized_result, admin)
    }

    #[test]
    #[should_panic]
    fn test_cannot_initialize_more_than_once() {
        let (env, contract_address, admin) = create_contract_variables();
        // env.register_contract(None, InstallmentPayment); the register contract method of the env has been deprecated

        let installment_payment_instance = InstallmentPaymentClient::new(&env, &contract_address); // the client with the new function return the instance of the contract
                                                                                                   // installment_payment_instance
                                                                                                   // let contract_instance = installment_payment_instance.initialize(&admin);

        installment_payment_instance.initialize(&admin);
        installment_payment_instance.initialize(&admin);
    }

    #[test]
    fn test_create_an_installment() {
        let (env, contract_address, mocked_address) = create_contract_variables();

        let installed_payment_instance = InstallmentPaymentClient::new(&env, &contract_address);

        let seller: Address = Address::generate(&env);
        let buyer: Address = Address::generate(&env);

        let deadline: u64 = env.ledger().timestamp() + 100;
        let description: String = String::from_str(&env, "agreement btw A and B");
        let amount: u128 = 80;
        let token: Address = Address::generate(&env);

        installed_payment_instance.create_installment_agreement(
            &seller,
            &buyer,
            &amount,
            &deadline,
            &mocked_address,
            &token,
            &description,
        );

        let optional_installment = installed_payment_instance.get_installment_agreement(&1);

        assert!(optional_installment.is_some());
    }

    #[test]
    fn test_create_multiple_installment() {
        let (env, contract_address, mocked_address) = create_contract_variables();

        let installed_payment_instance = InstallmentPaymentClient::new(&env, &contract_address);

        let seller: Address = Address::generate(&env);
        let buyer: Address = Address::generate(&env);

        let deadline: u64 = env.ledger().timestamp() + 100;
        let description: String = String::from_str(&env, "agreement btw A and B");
        let amount: u128 = 80;
        let token: Address = Address::generate(&env);

        installed_payment_instance.create_installment_agreement(
            &seller,
            &buyer,
            &amount,
            &deadline,
            &mocked_address,
            &token,
            &description,
        );

        // let optional_installment = installed_payment_instance.get_installment_agreement(&1);

        // assert!(optional_installment.is_some());

        installed_payment_instance.create_installment_agreement(
            &seller,
            &buyer,
            &100,
            &deadline,
            &mocked_address,
            &token,
            &description,
        );

        let optional_installment = installed_payment_instance.get_installment_agreement(&2);

        assert!(optional_installment.is_some());

        let installment = optional_installment.unwrap();

        assert_eq!(installment.total_amount, 100);

        // cannot get an agreement that the id does not exixst
        let optional_installment = installed_payment_instance.get_installment_agreement(&3);

        assert!(optional_installment.is_none());
    }

    #[test]
    // #[should_panic]
    fn test_accept_installment_agreement() {
        let (env, contract_address, mocked_address) = create_contract_variables();

        let installed_payment_instance = InstallmentPaymentClient::new(&env, &contract_address);

        let seller: Address = Address::generate(&env);
        let buyer: Address = Address::generate(&env);

        let deadline: u64 = env.ledger().timestamp() + 100;
        let description: String = String::from_str(&env, "agreement btw A and B");
        let amount: u128 = 80;
        let token: Address = Address::generate(&env);

        installed_payment_instance.create_installment_agreement(
            &seller,
            &buyer,
            &amount,
            &deadline,
            &mocked_address,
            &token,
            &description,
        );

        let optional_installment = installed_payment_instance.get_installment_agreement(&1);
        let installment = optional_installment.unwrap();

        assert_eq!(installment.is_accepted, false);

        installed_payment_instance.accept_installment_agreement(&seller, &true, &1);

        // get the state
        let optional_installment = installed_payment_instance.get_installment_agreement(&1);
        let installment = optional_installment.unwrap();

        assert_eq!(installment.is_accepted, true);
    }

    #[test]
    #[should_panic]
    fn test_only_seller_can_accept_agreement() {
        let (env, contract_address, mocked_address) = create_contract_variables();

        let installed_payment_instance = InstallmentPaymentClient::new(&env, &contract_address);

        let seller: Address = Address::generate(&env);
        let buyer: Address = Address::generate(&env);

        let deadline: u64 = env.ledger().timestamp() + 100;
        let description: String = String::from_str(&env, "agreement btw A and B");
        let amount: u128 = 80;
        let token: Address = Address::generate(&env);

        installed_payment_instance.create_installment_agreement(
            &seller,
            &buyer,
            &amount,
            &deadline,
            &mocked_address,
            &token,
            &description,
        );

        installed_payment_instance.accept_installment_agreement(&buyer, &true, &1);
    }

    #[test]
    #[should_panic]
    fn test_finalized_agreement() {
        let (env, contract_address, mocked_address) = create_contract_variables();

        let installed_payment_instance = InstallmentPaymentClient::new(&env, &contract_address);

        let seller: Address = Address::generate(&env);
        let buyer: Address = Address::generate(&env);

        let deadline: u64 = env.ledger().timestamp() + 100;
        let description: String = String::from_str(&env, "agreement btw A and B");
        let amount: u128 = 80;
        let token: Address = Address::generate(&env);

        installed_payment_instance.create_installment_agreement(
            &seller,
            &buyer,
            &amount,
            &deadline,
            &mocked_address,
            &token,
            &description,
        );

        installed_payment_instance.finalize_agreement(&1, &buyer);
    }

    fn create_token(env: &Env, admin: &Address) -> (Address, StellarAssetClient<'static>) {
        let client = env.register_stellar_asset_contract_v2(admin.clone());
        (
            client.address(),
            token::StellarAssetClient::new(&env, &client.address()),
        )
    }

    #[test]
    fn test_pay_on_installment() {
        let (env, contract_address, mocked_address) = create_contract_variables();
        let installed_payment_instance = InstallmentPaymentClient::new(&env, &contract_address);
        let (token, token_client) = create_token(&env, &mocked_address);

        let seller: Address = Address::generate(&env);
        let buyer: Address = Address::generate(&env);

        token_client.mint(&buyer, &200);

        let deadline: u64 = env.ledger().timestamp() + 100;
        let description: String = String::from_str(&env, "agreement btw A and B");
        let amount: u128 = 80;
        // let token: Address = Address::generate(&env);

        installed_payment_instance.create_installment_agreement(
            &seller,
            &buyer,
            &amount,
            &deadline,
            &mocked_address,
            &token,
            &description,
        );

        // accept agreement
        installed_payment_instance.accept_installment_agreement(&seller, &true, &1);

        installed_payment_instance.pay_on_installment(&buyer, &30, &1);

        let optional_installment = installed_payment_instance.get_installment_agreement(&1);
        let installment = optional_installment.unwrap();

        log!(&env, "installment ::: {:?}", installment);

        assert_eq!(installment.amount_paid, 30);

        installed_payment_instance.pay_on_installment(&buyer, &20, &1);
        installed_payment_instance.pay_on_installment(&buyer, &30, &1);

        let optional_installment = installed_payment_instance.get_installment_agreement(&1);
        let installment = optional_installment.unwrap();
        assert_eq!(installment.amount_paid, 80);
        assert_eq!(installment.paid_history.get(1).unwrap().amount, 20);

        assert_eq!(installment.paid_history.len(), 3);

        log!(&env, "installment ::: {:?}", installment);

        log!(&env, "time ::: {}", &env.ledger().timestamp());
        log!(&env, "increated ::: {}", &env.ledger().timestamp());
    }

    #[test]
    #[should_panic(expected = "agreement past stipulated deadline")]
    fn test_pay_on_installment_past_deadline() {
        let (env, contract_address, mocked_address) = create_contract_variables();
        let installed_payment_instance = InstallmentPaymentClient::new(&env, &contract_address);
        let (token, token_client) = create_token(&env, &mocked_address);

        let seller: Address = Address::generate(&env);
        let buyer: Address = Address::generate(&env);

        token_client.mint(&buyer, &200);

        let deadline: u64 = env.ledger().timestamp() + 100;
        let description: String = String::from_str(&env, "agreement btw A and B");
        let amount: u128 = 80;
        // let token: Address = Address::generate(&env);

        installed_payment_instance.create_installment_agreement(
            &seller,
            &buyer,
            &amount,
            &deadline,
            &mocked_address,
            &token,
            &description,
        );

        // accept agreement
        installed_payment_instance.accept_installment_agreement(&seller, &true, &1);

        env.ledger().set_timestamp(200);
        installed_payment_instance.pay_on_installment(&buyer, &30, &1);
    }

    #[test]
    fn test_cancel_and_refund() {
        let (env, contract_address, mocked_address) = create_contract_variables();
        let installed_payment_instance = InstallmentPaymentClient::new(&env, &contract_address);
        let (token, token_client) = create_token(&env, &mocked_address);

        let seller: Address = Address::generate(&env);
        let buyer: Address = Address::generate(&env);

        token_client.mint(&buyer, &200);

        let deadline: u64 = env.ledger().timestamp() + 100;
        let description: String = String::from_str(&env, "agreement btw A and B");
        let amount: u128 = 80;
        // let token: Address = Address::generate(&env);

        installed_payment_instance.create_installment_agreement(
            &seller,
            &buyer,
            &amount,
            &deadline,
            &mocked_address,
            &token,
            &description,
        );

        installed_payment_instance.accept_installment_agreement(&seller, &true, &1);

        installed_payment_instance.pay_on_installment(&buyer, &40, &1);

        let token_contract = token::TokenClient::new(&env, &token);
        let buyer_balance = token_contract.balance(&buyer);

        assert_eq!(buyer_balance, 160);

        installed_payment_instance.cancel_and_refund_agreement(&seller, &1);

        let buyer_balance = token_contract.balance(&buyer);

        assert_eq!(buyer_balance, 200);

        let optional_installment = installed_payment_instance.get_installment_agreement(&1);

        assert_eq!(optional_installment.unwrap().is_canceled, true);
    }

    #[test]
    fn test_finalize_installment() {
        let (env, contract_address, mocked_address) = create_contract_variables();
        let installed_payment_instance = InstallmentPaymentClient::new(&env, &contract_address);
        let (token, token_client) = create_token(&env, &mocked_address);

        let seller: Address = Address::generate(&env);
        let buyer: Address = Address::generate(&env);

        token_client.mint(&buyer, &200);

        let deadline: u64 = env.ledger().timestamp() + 100;
        let description: String = String::from_str(&env, "agreement btw A and B");
        let amount: u128 = 80;
        // let token: Address = Address::generate(&env);

        installed_payment_instance.create_installment_agreement(
            &seller,
            &buyer,
            &amount,
            &deadline,
            &mocked_address,
            &token,
            &description,
        );

        installed_payment_instance.accept_installment_agreement(&seller, &true, &1);

        installed_payment_instance.pay_on_installment(&buyer, &70, &1);
        installed_payment_instance.pay_on_installment(&buyer, &17, &1);

        installed_payment_instance.finalize_agreement(&1, &buyer);

        let installment_agreement = installed_payment_instance
            .get_installment_agreement(&1)
            .unwrap();

        let token_contract = token::TokenClient::new(&env, &token);

        let amount = &(installment_agreement.total_amount as i128);
        let seller_balance = &token_contract.balance(&seller);

        assert_eq!(seller_balance, amount);

        let agree_ment = installed_payment_instance
            .get_installment_agreement(&1)
            .unwrap();
        assert_eq!(agree_ment.is_finalized, true);
        // env.ledger().set_timestamp(300);
    }
}
