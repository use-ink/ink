#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contract_xcm {
    use ink::{
        env::Error as EnvError,
        xcm::prelude::*,
    };

    /// A trivial contract used to exercise XCM API.
    #[ink(storage)]
    #[derive(Default)]
    pub struct ContractXcm;

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum RuntimeError {
        XcmExecuteFailed,
        XcmSendFailed,
    }

    impl From<EnvError> for RuntimeError {
        fn from(e: EnvError) -> Self {
            use ink::env::ReturnErrorCode;
            match e {
                EnvError::ReturnError(ReturnErrorCode::XcmExecutionFailed) => {
                    RuntimeError::XcmExecuteFailed
                }
                EnvError::ReturnError(ReturnErrorCode::XcmSendFailed) => {
                    RuntimeError::XcmSendFailed
                }
                _ => panic!("Unexpected error from `pallet-contracts`."),
            }
        }
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
            let asset: Asset = (Here, value).into();
            let beneficiary = AccountId32 {
                network: None,
                id: *receiver.as_ref(),
            };

            let message: Xcm<()> = Xcm::builder()
                .withdraw_asset(asset.clone().into())
                .buy_execution(asset.clone(), Unlimited)
                .deposit_asset(asset.into(), beneficiary.into())
                .build();

            self.env()
                .xcm_execute(&VersionedXcm::V4(message))
                .map_err(Into::into)
        }

        /// Transfer some funds on the relay chain via XCM from the contract's derivative
        /// account to the caller's account.
        ///
        /// Fails if:
        ///  - called in the off-chain environment
        ///  - the chain is not configured to support XCM
        ///  - the XCM program executed failed (e.g contract doesn't have enough balance)
        #[ink(message)]
        pub fn send_funds(
            &mut self,
            value: Balance,
            fee: Balance,
        ) -> Result<XcmHash, RuntimeError> {
            let destination: Location = Parent.into();
            let asset: Asset = (Here, value).into();
            let beneficiary = AccountId32 {
                network: None,
                id: *self.env().caller().as_ref(),
            };

            let message: Xcm<()> = Xcm::builder()
                .withdraw_asset(asset.clone().into())
                .buy_execution((Here, fee).into(), WeightLimit::Unlimited)
                .deposit_asset(asset.into(), beneficiary.into())
                .build();

            let hash = self.env().xcm_send(
                &VersionedLocation::V4(destination),
                &VersionedXcm::V4(message),
            )?;

            Ok(hash)
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use frame_support::{
            sp_runtime::AccountId32,
            traits::tokens::currency::Currency,
        };
        use ink::{
            env::{
                test::default_accounts,
                DefaultEnvironment,
            },
            primitives::AccountId,
        };
        use ink_e2e::{
            preset::mock_network::{
                self,
                primitives::{
                    CENTS,
                    UNITS,
                },
                MockNetworkSandbox,
            },
            ChainBackend,
            ContractsBackend,
        };
        use mock_network::{
            parachain::estimate_message_fee,
            parachain_account_sovereign_account_id,
            relay_chain,
            Relay,
            TestExt,
        };

        use super::*;

        /// The contract will be given 1000 tokens during instantiation.
        pub const CONTRACT_BALANCE: u128 = 1_000 * UNITS;

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(backend(runtime_only(sandbox = MockNetworkSandbox)))]
        async fn xcm_execute_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = ContractXcmRef::new();
            let contract = client
                .instantiate("contract_xcm", &ink_e2e::alice(), &mut constructor)
                .value(CONTRACT_BALANCE)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<ContractXcm>();

            let receiver: AccountId = default_accounts::<DefaultEnvironment>().bob;

            let contract_balance_before = client
                .free_balance(contract.account_id)
                .await
                .expect("Failed to get account balance");
            let receiver_balance_before = client
                .free_balance(receiver)
                .await
                .expect("Failed to get account balance");

            // when
            let amount = 1000 * CENTS;
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

            assert_eq!(contract_balance_before, contract_balance_after + amount);
            assert_eq!(receiver_balance_before, receiver_balance_after - amount);

            Ok(())
        }

        #[ink_e2e::test(backend(runtime_only(sandbox = MockNetworkSandbox)))]
        async fn incomplete_xcm_execute_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let mut constructor = ContractXcmRef::new();
            let contract = client
                .instantiate("contract_xcm", &ink_e2e::alice(), &mut constructor)
                .value(CONTRACT_BALANCE)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<ContractXcm>();

            // This will fail since we have insufficient balance
            let transfer_message = call_builder.transfer_through_xcm(
                default_accounts::<DefaultEnvironment>().bob,
                CONTRACT_BALANCE + 1,
            );

            let call_res = client
                .call(&ink_e2e::alice(), &transfer_message)
                .submit()
                .await?
                .return_value();

            assert!(matches!(call_res, Err(RuntimeError::XcmExecuteFailed)));
            Ok(())
        }

        #[ink_e2e::test(backend(runtime_only(sandbox = MockNetworkSandbox)))]
        async fn xcm_send_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            let mut constructor = ContractXcmRef::new();
            let contract = client
                .instantiate("contract_xcm", &ink_e2e::alice(), &mut constructor)
                .value(CONTRACT_BALANCE)
                .submit()
                .await
                .expect("instantiate failed");

            Relay::execute_with(|| {
                let sovereign_account = parachain_account_sovereign_account_id(
                    1u32,
                    AccountId32::from(contract.account_id.0),
                );

                // Fund the contract's derivative account, so we can use it as a sink, to
                // transfer funds to the caller.
                relay_chain::Balances::make_free_balance_be(
                    &sovereign_account,
                    CONTRACT_BALANCE,
                );
            });

            let amount = 1000 * CENTS;
            let fee = estimate_message_fee(4);

            let mut call_builder = contract.call_builder::<ContractXcm>();
            let message = call_builder.send_funds(amount, fee);
            let call_res = client.call(&ink_e2e::alice(), &message).submit().await?;
            assert!(call_res.return_value().is_ok());

            Relay::execute_with(|| {
                let alice = AccountId32::from(ink_e2e::alice().public_key().0);
                assert_eq!(relay_chain::Balances::free_balance(&alice), amount - fee);
            });

            Ok(())
        }
    }
}
