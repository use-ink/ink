#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contract_xcm {
    use ink::xcm::prelude::*;

    /// A contract demonstrating usage of the XCM API.
    #[ink(storage)]
    #[derive(Default)]
    pub struct ContractXcm;

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum RuntimeError {
        XcmExecuteFailed,
        XcmSendFailed,
    }

    impl ContractXcm {
        /// The constructor is `payable`, so that during instantiation it can be given
        /// some tokens that will be further transferred when transferring funds through
        /// XCM.
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Tries to transfer `value` from the contract's balance to `receiver`.
        ///
        /// Fails if:
        ///  - called in the off-chain environment
        ///  - the chain is not configured to support XCM
        ///  - the XCM program executed failed (e.g contract doesn't have enough balance)
        #[ink(message)]
        pub fn transfer_through_xcm(
            &mut self,
            receiver: AccountId,
            value: Balance,
        ) -> Result<(), RuntimeError> {
            let asset: Asset = (Parent, value).into();
            let beneficiary = AccountId32 {
                network: None,
                id: *receiver.as_ref(),
            };

            let message: ink::xcm::v5::Xcm<()> = Xcm::builder()
                .withdraw_asset(asset.clone())
                .buy_execution(asset.clone(), Unlimited)
                .deposit_asset(asset, beneficiary)
                .build();
            let msg = VersionedXcm::V5(message);

            let weight = self.env().xcm_weigh(&msg).expect("weight should work");

            self.env()
                .xcm_execute(&msg, weight)
                .map_err(|_| RuntimeError::XcmExecuteFailed)
        }

        /// Transfer some funds to the relay chain via XCM from the contract's derivative
        /// account to the caller's account.
        ///
        /// Fails if:
        ///  - called in the off-chain environment
        ///  - the chain is not configured to support XCM
        ///  - the XCM program executed failed (e.g. contract doesn't have enough balance)
        #[ink(message)]
        pub fn send_funds(
            &mut self,
            value: Balance,
            fee: Balance,
        ) -> Result<(), RuntimeError> {
            // The destination of the XCM message. Assuming we run the contract
            // on a parachain, the parent will be the relay chain.
            let destination: ink::xcm::v5::Location = ink::xcm::v5::Parent.into();

            // The asset to be sent, since we are sending the XCM to the relay chain,
            // this represents `value` amount of the relay chain's native asset.
            let asset: Asset = (Here, value).into();

            // The beneficiary of the asset.
            // Here, the beneficiary is the caller's account on the relay chain.
            let caller_account_id = self.env().to_account_id(self.env().caller());
            let beneficiary = AccountId32 {
                network: None,
                id: caller_account_id.0,
            };

            // Create an XCM message
            let message: Xcm<()> = Xcm::builder()
                // Withdraw the asset from the origin (the sovereign account of the
                // contract on the relay chain)
                .withdraw_asset(asset.clone())

                // Buy execution to pay the fee on the relay chain
                .buy_execution((Here, fee), WeightLimit::Unlimited)

                // Deposit the asset to the caller's account on the relay chain
                .deposit_asset(asset, beneficiary)
                .build();

            // Send the constructed XCM message to the relay chain.
            self.env()
                .xcm_send(
                    &VersionedLocation::V5(destination),
                    &VersionedXcm::V5(message),
                )
                .map_err(|_| RuntimeError::XcmSendFailed)
        }

        #[ink(message)]
        pub fn reserve_transfer(
            &mut self,
            amount: Balance,
            fee: Balance,
        ) -> Result<(), RuntimeError> {
            // The beneficiary of the transfer.
            // Here, the beneficiary is the caller's account on the relay chain.
            let caller_account_id = self.env().to_account_id(self.env().caller());
            let beneficiary: Location = AccountId32 {
                network: None,
                id: caller_account_id.0,
            }
            .into();

            // Create an XCM message.
            let message: Xcm<()> = Xcm::builder_unsafe()
                // Withdraw the relay's native token derivative from the
                // contract's account.
                .withdraw_asset((Parent, amount))

                // The `initiate_reserve_withdraw` instruction takes the
                // derivative token from the holding register and burns it.
                // It then sends the nested XCM to the reserve in this
                // example, the relay chain.
                // Upon receiving the XCM, the reserve will withdraw the
                // asset from our chain's sovereign account, and deposit
                // on the caller's account.
                .initiate_reserve_withdraw(
                    All,
                    Parent,
                    Xcm::builder_unsafe()
                        .buy_execution((Here, fee), Unlimited)
                        .deposit_asset(All, beneficiary)
                        .build(),
                )
                .build();

            let msg = VersionedXcm::V5(message);
            let weight = self.env().xcm_weigh(&msg).expect("`xcm_weigh` failed");
            self.env()
                .xcm_execute(&msg, weight)
                .map_err(|_| RuntimeError::XcmExecuteFailed)
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink::primitives::AccountId;
        use ink_e2e::{
            ChainBackend,
            ContractsBackend,
        };

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn xcm_execute_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = ContractXcmRef::new();
            let contract = client
                .instantiate("contract_xcm", &ink_e2e::alice(), &mut constructor)
                .value(100_000_000_000)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<ContractXcm>();

            let receiver = AccountId::from(ink_e2e::bob().public_key().0);

            let contract_balance_before = client
                .free_balance(contract.account_id)
                .await
                .expect("Failed to get account balance");
            let receiver_balance_before = client
                .free_balance(receiver)
                .await
                .expect("Failed to get account balance");

            // when
            let amount = 100_000_000;
            let transfer_message = call_builder.transfer_through_xcm(receiver, amount);
            let call_res = client
                .call(&ink_e2e::alice(), &transfer_message)
                .submit()
                .await
                .expect("call failed");
            assert!(call_res.return_value().is_ok());

            // then
            let contract_balance_after = client
                .free_balance(contract.account_id)
                .await
                .expect("Failed to get account balance");
            let receiver_balance_after = client
                .free_balance(receiver)
                .await
                .expect("Failed to get account balance");

            assert_eq!(contract_balance_after, contract_balance_before - amount);
            assert_eq!(receiver_balance_after, receiver_balance_before + amount);

            Ok(())
        }

        #[ink_e2e::test]
        async fn xcm_execute_failure_detection_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // todo @cmichi: This sleep is necessary until we have our `ink-node`
            // support a parachain/relaychain setup. For the moment we use the
            // Rococo runtime for testing the examples locally. That runtime
            // only has Alice and Bob endowed. Due to the nature of the tests
            // we have to use Alice for sending the transactions. If the tests
            // run at the same time, we'll get an error because the nonce
            // of Alice is the same for all transactions.
            std::thread::sleep(std::time::Duration::from_secs(10));

            // given
            let mut constructor = ContractXcmRef::new();
            let contract = client
                .instantiate("contract_xcm", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<ContractXcm>();

            // when
            let receiver = AccountId::from(ink_e2e::bob().public_key().0);
            let amount = u128::MAX;
            let transfer_message = call_builder.transfer_through_xcm(receiver, amount);

            // then
            let call_res = client
                .call(&ink_e2e::alice(), &transfer_message)
                .submit()
                .await;
            assert!(call_res.is_err());

            let expected = "revert: XCM execute failed: message may be invalid or execution constraints not satisfied";
            assert!(format!("{:?}", call_res).contains(expected));

            Ok(())
        }

        #[ink_e2e::test]
        async fn xcm_send_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // todo @cmichi: This sleep is necessary until we have our `ink-node`
            // support a parachain/relaychain setup. For the moment we use the
            // Rococo runtime for testing the examples locally. That runtime
            // only has Alice and Bob endowed. Due to the nature of the tests
            // we have to use Alice for sending the transactions. If the tests
            // run at the same time, we'll get an error because the nonce
            // of Alice is the same for all transactions.
            std::thread::sleep(std::time::Duration::from_secs(30));

            // given
            let mut constructor = ContractXcmRef::new();
            let contract = client
                .instantiate("contract_xcm", &ink_e2e::alice(), &mut constructor)
                .value(100_000_000_000)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<ContractXcm>();

            let contract_balance_before = client
                .free_balance(contract.account_id)
                .await
                .expect("Failed to get account balance");

            // when
            let amount = 100_000_000;
            let transfer_message = call_builder.send_funds(amount, amount / 2);
            let call_res = client
                .call(&ink_e2e::alice(), &transfer_message)
                .submit()
                .await
                .expect("call failed");
            assert!(call_res.return_value().is_ok());

            // then
            let contract_balance_after = client
                .free_balance(contract.account_id)
                .await
                .expect("Failed to get account balance");

            assert!(
                contract_balance_after <= contract_balance_before - amount - (amount / 2)
            );

            Ok(())
        }
    }
}
