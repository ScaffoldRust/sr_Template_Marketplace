use soroban_sdk::{
    contract, contractimpl, symbol_short,
    token::{self, TokenClient},
    Address, Env, String, Symbol,
};

use crate::errors::errors::*;
use crate::storage::{contracts::*, storage::*};

const ADMIN: Symbol = symbol_short!("i_p_admin"); // length cannot be more than 9, hence, i = installment, p = payment,

#[contract]
pub struct InstallmentPayment;

#[contractimpl]
impl InstallmentPayment {
    // initialize the contract
    // @params: env of type Env is the environment variable from soroban
    // @params: admin which would act as the owner of this contract
    pub fn initialize(env: Env, admin: Address) -> Result<Address, ContractError> {
        // we only want to create just one admin inn this contract
        let admin_exist: bool = env.storage().persistent().has(&ADMIN);
        // println!("admin exist {:p}, {:#}", &admin_exist, &admin_exist);
        if admin_exist {
            return Err(ContractError::AlreadyInstantiated);
            // panic!("");
        }
        // check auth
        admin.require_auth();
        env.storage().persistent().set(&ADMIN, &admin);
        Ok(admin)
    }

    // since it is the buyer that want to pay on installment , it best we allow buyer to create the agreement then choose list pf Arbitrator provided by the platform
    // if the seller is satisfied with the agreement , the seller accepts/ agree to the agreement which then buyer can start making deposits
    pub fn create_installment_agreement(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u128,
        deadline: u64,
        arbitrator: Address,
        token: Address,
        description: String,
    ) -> Result<bool, ContractError> {
        buyer.require_auth();

        // confirm the deadline and amount
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }

        // ensure buyer is not the seller
        if &buyer == &seller {
            return Err(ContractError::DuplicateUsers);
        }

        if &buyer == &arbitrator || &seller == &arbitrator {
            return Err(ContractError::ArbitratorNotAllowed);
        }

        if env.ledger().timestamp() > (env.ledger().timestamp() + deadline) {
            return Err(ContractError::InvalidTimestamp);
        }

        // generate the agreement id
        let agreement_id: u128 = get_agreement_id(&env);
        let new_agreement_id: u128 = agreement_id + 1;

        // create the agreement
        let install_agreement: InstallmentAgreement = InstallmentAgreement::new(
            &env,
            new_agreement_id,
            buyer,
            seller,
            amount,
            deadline,
            arbitrator,
            description,
            token,
        );

        //save the agreement
        save_new_agreement_id(&env, new_agreement_id);
        save_installment_agreement(&env, new_agreement_id, install_agreement);

        env.events()
            .publish(("installment_agreement_created",), new_agreement_id);

        Ok(true)
    }

    pub fn pay_on_installment(
        env: Env,
        buyer_address: Address,
        installment_amount: u128,
        agreement_id: u128,
    ) -> Result<bool, ContractError> {
        buyer_address.require_auth();

        let installment_agreement_optional: Option<InstallmentAgreement> =
            get_installment_agreement(&env, agreement_id);

        if &installment_agreement_optional.is_none() == &true {
            return Err(ContractError::AgreementNotFOund);
        }

        let mut installment_agreement: InstallmentAgreement =
            installment_agreement_optional.unwrap();

        assert!(
            &installment_agreement.is_accepted,
            "agreement not accepted yet"
        );
        assert!(
            !&installment_agreement.is_canceled,
            "agreement has been cancelled"
        );
        assert!(
            !&installment_agreement.is_finalized,
            "agreement has been finalized"
        );
        assert!(
            &installment_agreement.deadline > &env.ledger().timestamp(),
            "agreement past stipulated deadline"
        );

        assert!(installment_amount > 0, "amount cannot be zero");

        let token_address: &Address = &installment_agreement.token;

        // create the token client
        let token_contract: TokenClient = token::TokenClient::new(&env, &token_address);
        let user_balance: i128 = token_contract.balance(&buyer_address);

        assert!(
            user_balance >= installment_amount as i128,
            "insufficient balance"
        );

        token_contract.transfer(
            &buyer_address,
            &env.current_contract_address(),
            &(installment_amount as i128),
        );

        installment_agreement
            .update_installment_agreement_payment_and_history(&env, installment_amount);

        save_installment_agreement(&env, agreement_id, installment_agreement);

        env.events().publish(
            ("installment_payment_made",),
            (&agreement_id, &buyer_address, &installment_amount),
        );

        Ok(true)
    }

    pub fn finalize_agreement(
        env: Env,
        agreement_id: u128,
        user: Address,
    ) -> Result<bool, ContractError> {
        user.require_auth();
        // this checks if the deadline has been met or the total amout has been met
        let installment_agreement_optional: Option<InstallmentAgreement> =
            get_installment_agreement(&env, agreement_id);

        if installment_agreement_optional.is_none() {
            return Err(ContractError::AgreementNotFOund);
        }

        let mut installment_agreement: InstallmentAgreement =
            installment_agreement_optional.unwrap();

        // on the buyer or the seller can finalize
        assert!(
            user == installment_agreement.buyer || user == installment_agreement.seller,
            ""
        );

        assert!(
            installment_agreement.is_accepted,
            "agreement not yet accepted"
        );
        assert!(
            !installment_agreement.is_finalized,
            "agreement has been finalized"
        );
        assert!(
            !installment_agreement.is_canceled,
            "agreement has been canceled"
        );

        // let current_time = env.ledger().timestamp();
        assert!(
            installment_agreement.amount_paid >= installment_agreement.total_amount,
            // || current_time > installment_agreement.deadline,
            "agreed amount not met yet"
        );

        // send to the seller
        let token_contract = token::TokenClient::new(&env, &installment_agreement.token);

        token_contract.transfer(
            &env.current_contract_address(),
            &installment_agreement.seller,
            &(installment_agreement.total_amount as i128),
        );

        installment_agreement.finalize();

        save_installment_agreement(&env, agreement_id, installment_agreement);
        env.events()
            .publish(("agreement_published",), (&agreement_id, &user));

        Ok(true)
    }

    pub fn accept_installment_agreement(
        env: Env,
        seller: Address,
        accept_agreement: bool,
        agreement_id: u128,
    ) -> Result<bool, ContractError> {
        seller.require_auth();

        let installment_agreement_optional: Option<InstallmentAgreement> =
            get_installment_agreement(&env, agreement_id);

        if installment_agreement_optional.is_none() {
            return Err(ContractError::InvalidAgreementId);
        }
        let mut installment_agreement: InstallmentAgreement =
            installment_agreement_optional.unwrap();

        if &seller != &installment_agreement.seller {
            return Err(ContractError::NotAuthorized);
        }

        assert!(
            !&installment_agreement.is_accepted,
            "cannot carry out action agreement has been accepted"
        );
        assert!(
            !&installment_agreement.is_canceled,
            "cannot carry out action as agreement has previously been canceled"
        );
        assert!(
            !&installment_agreement.is_finalized,
            "cannot carry out action as agreement has been previously finalized"
        );

        installment_agreement.accept_agreement(accept_agreement);

        save_installment_agreement(&env, agreement_id, installment_agreement);

        env.events().publish(
            ("accept_agreement",),
            (&agreement_id, &seller, &accept_agreement),
        );
        Ok(true)
    }

    // only seller can cancel the agreement
    pub fn cancel_and_refund_agreement(
        env: Env,
        address: Address,
        agreement_id: u128,
    ) -> Result<bool, ContractError> {
        address.require_auth();

        let installment_agreement_optional: Option<InstallmentAgreement> =
            get_installment_agreement(&env, agreement_id);

        if installment_agreement_optional.is_none() == true {
            return Err(ContractError::AgreementNotFOund);
        }

        let mut installment_agreement: InstallmentAgreement =
            installment_agreement_optional.unwrap();

        assert!(installment_agreement.is_accepted, "");
        assert!(!installment_agreement.is_canceled, "");
        assert!(!installment_agreement.is_finalized, "");

        assert_eq!(
            installment_agreement.seller, address,
            "only the seller can cancel agreement"
        );

        // change the state to true
        installment_agreement.cancel_agreement();

        // check the total amount paid by the buyer and refund
        let total_installment_amount_paid: u128 = installment_agreement.amount_paid;

        if total_installment_amount_paid > 0 {
            // check if the amount paid is greater than 0
            let token_contract: TokenClient =
                token::TokenClient::new(&env, &installment_agreement.token);

            // call on the token client

            token_contract.transfer(
                &env.current_contract_address(),
                &installment_agreement.buyer,
                &(total_installment_amount_paid as i128),
            );
        }

        //save to the storage
        save_installment_agreement(&env, agreement_id, installment_agreement);

        env.events()
            .publish(("cancel_and_refund",), (&agreement_id, &address));
        Ok(true)
    }

    pub fn get_installment_agreement(env: Env, agreement_id: u128) -> Option<InstallmentAgreement> {
        // Err(String::from_str(&env, ""))
        get_installment_agreement(&env, agreement_id)
    }
}
