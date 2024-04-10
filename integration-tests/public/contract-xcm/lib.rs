#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contract_xcm {
    use ink::{
        env::Error as EnvError,
        prelude::*,
        xcm::{
            v4::prelude::*,
            VersionedLocation,
            VersionedXcm,
        },
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
            self.env()
                .xcm_execute(&VersionedXcm::V4(Xcm::<()>(vec![
                    WithdrawAsset(vec![(Here, value).into()].into()),
                    DepositAsset {
                        assets: All.into(),
                        beneficiary: AccountId32 {
                            network: None,
                            id: *receiver.as_ref(),
                        }
                        .into(),
                    },
                ])))
                .map_err(Into::into)
        }

        /// Sends an lock some funds to the relay chain.
        ///
        /// Fails if:
        ///  - called in the off-chain environment
        ///  - the chain is not configured to support XCM
        ///  - the XCM program executed failed (e.g contract doesn't have enough balance)
        #[ink(message)]
        pub fn lock_funds_to_relay(
            &mut self,
            value: Balance,
            fee: Balance,
        ) -> Result<XcmHash, RuntimeError> {
            let hash = self.env().xcm_send(
                &VersionedLocation::V4(Location::from(Parent)),
                &VersionedXcm::V4(Xcm::<()>(vec![
                    WithdrawAsset((Here, fee).into()),
                    BuyExecution {
                        fees: (Here, fee).into(),
                        weight_limit: WeightLimit::Unlimited,
                    },
                    LockAsset {
                        asset: (Here, value).into(),
                        unlocker: (Parachain(1)).into(),
                    },
                ])),
            )?;
            Ok(hash)
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
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
                MockNetworkSandbox,
            },
            ChainBackend,
            ContractsBackend,
        };

        use super::*;

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        /// The base number of indivisible units for balances on the
        /// `substrate-contracts-node`.
        const UNIT: Balance = 1_000_000_000_000;

        /// The contract will be given 1000 tokens during instantiation.
        const CONTRACT_BALANCE: Balance = 10 * UNIT;

        /// The receiver will get enough funds to have the required existential deposit.
        ///
        /// If your chain has this threshold higher, increase the transfer value.
        const TRANSFER_VALUE: Balance = 1 * UNIT;

        #[ink_e2e::test(backend(runtime_only(sandbox = MockNetworkSandbox)))]
        async fn transfer_with_xcm_execute_works<Client: E2EBackend>(
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
            let transfer_message =
                call_builder.transfer_through_xcm(receiver, TRANSFER_VALUE);

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

            assert_eq!(
                contract_balance_before,
                contract_balance_after + TRANSFER_VALUE
            );
            assert_eq!(
                receiver_balance_before,
                receiver_balance_after - TRANSFER_VALUE
            );

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
                .dry_run()
                .await?
                .return_value();

            assert!(matches!(call_res, Err(RuntimeError::XcmExecuteFailed)));
            Ok(())
        }

        #[ink_e2e::test(backend(runtime_only(sandbox = MockNetworkSandbox)))]
        async fn xcm_send_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            use frame_support::traits::{
                fungibles::Mutate,
                tokens::currency::Currency,
            };
            use mock_network::{
                parachain,
                parachain_account_sovereign_account_id,
                relay_chain,
                ParaA,
                Relay,
                TestExt,
                INITIAL_BALANCE,
            };
            use pallet_balances::{
                BalanceLock,
                Reasons,
            };

            let mut constructor = ContractXcmRef::new();
            let contract = client
                .instantiate("contract_xcm", &ink_e2e::alice(), &mut constructor)
                .value(CONTRACT_BALANCE)
                .submit()
                .await
                .expect("instantiate failed");

            let account_id: &[u8; 32] = contract.account_id.as_ref();
            let account_id: [u8; 32] = account_id.clone();
            let account_id = account_id.into();

            ParaA::execute_with(|| {
                parachain::Balances::make_free_balance_be(&account_id, INITIAL_BALANCE);
                parachain::Assets::mint_into(0u32.into(), &account_id, INITIAL_BALANCE)
                    .unwrap();
            });

            Relay::execute_with(|| {
                let sovereign_account =
                    parachain_account_sovereign_account_id(1u32, account_id.clone());
                relay_chain::Balances::make_free_balance_be(
                    &sovereign_account,
                    INITIAL_BALANCE,
                );
            });

            let mut call_builder = contract.call_builder::<ContractXcm>();
            let message = call_builder.lock_funds_to_relay(TRANSFER_VALUE, 8_000);
            let call_res = client.call(&ink_e2e::alice(), &message).submit().await?;

            assert!(call_res.return_value().is_ok());

            // Check if the funds are locked on the relay chain.
            Relay::execute_with(|| {
                assert_eq!(
                    relay_chain::Balances::locks(
                        &parachain_account_sovereign_account_id(1, account_id)
                    ),
                    vec![BalanceLock {
                        id: *b"py/xcmlk",
                        amount: TRANSFER_VALUE,
                        reasons: Reasons::All
                    }]
                );
            });

            Ok(())
        }
    }
}
