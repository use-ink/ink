#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod subxt {
    pub mod ext {
        pub use scale as codec;
    }

    pub mod utils {
        pub use primitive_types::H256;
        pub use sp_runtime::{
            AccountId32,
            MultiAddress,
        };
    }
}

#[subxt_macro::subxt(
    runtime_metadata_url = "ws://localhost:9944",
    runtime_types_only,
    no_default_derives,
    derive_for_all_types = "crate::subxt::ext::codec::Encode",
    attributes_for_all_types = "#[codec(crate = crate::subxt::ext::codec)]",
    crate = "crate::subxt"
)]
mod substrate {}

#[ink::contract]
mod runtime_call {
    use crate::substrate::runtime_types::{
        contracts_node_runtime::RuntimeCall,
        pallet_balances::pallet::Call as BalanceCall,
    };
    use ink::env::Error as EnvError;

    /// A trivial contract with a single message, that uses `call-runtime` API for
    /// performing native token transfer.
    #[ink(storage)]
    #[derive(Default)]
    pub struct RuntimeCaller;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum RuntimeError {
        CallRuntimeFailed,
    }

    impl From<EnvError> for RuntimeError {
        fn from(e: EnvError) -> Self {
            match e {
                EnvError::CallRuntimeFailed => RuntimeError::CallRuntimeFailed,
                _ => panic!("Unexpected error from `pallet-contracts`."),
            }
        }
    }

    impl RuntimeCaller {
        /// The constructor is `payable`, so that during instantiation it can be given
        /// some tokens that will be further transferred with
        /// `transfer_through_runtime` message.
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Tries to transfer `value` from the contract's balance to `receiver`.
        ///
        /// Fails if:
        ///  - called in the off-chain environment
        ///  - the chain doesn't allow `call-runtime` API (`UnsafeUnstableInterface` is
        ///    turned off)
        ///  - the chain forbids contracts to call `Balances::transfer` (`CallFilter` is
        ///    too restrictive)
        ///  - after the transfer, `receiver` doesn't have at least existential deposit
        ///  - the contract doesn't have enough balance
        #[ink(message)]
        pub fn transfer_through_runtime(
            &mut self,
            receiver: AccountId,
            value: Balance,
        ) -> Result<(), RuntimeError> {
            let receiver: [u8; 32] = *receiver.as_ref();
            let receiver: sp_runtime::AccountId32 = receiver.into();
            self.env()
                .call_runtime(&RuntimeCall::Balances(BalanceCall::transfer {
                    dest: receiver.into(),
                    value,
                }))
                .map_err(Into::into)
        }

        /// Tries to trigger `call_runtime` API with rubbish data.
        ///
        /// # Note
        ///
        /// This message is for testing purposes only.
        #[ink(message)]
        pub fn call_nonexistent_extrinsic(&mut self) -> Result<(), RuntimeError> {
            self.env().call_runtime(&()).map_err(Into::into)
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;

        use ink::{
            env::{
                test::default_accounts,
                DefaultEnvironment,
            },
            primitives::AccountId,
        };
        use ink_e2e::build_message;

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        /// The base number of indivisible units for balances on the
        /// `substrate-contracts-node`.
        const UNIT: Balance = 1_000_000_000_000;

        /// The contract will be given 1000 tokens during instantiation.
        #[cfg(feature = "permissive-node")]
        const CONTRACT_BALANCE: Balance = 1_000 * UNIT;

        /// The receiver will get enough funds to have the required existential deposit.
        ///
        /// If your chain has this threshold higher, increase the transfer value.
        const TRANSFER_VALUE: Balance = 1 / 10 * UNIT;

        /// An amount that is below the existential deposit, so that a transfer to an
        /// empty account fails.
        ///
        /// Must not be zero, because such an operation would be a successful no-op.
        #[cfg(feature = "permissive-node")]
        const INSUFFICIENT_TRANSFER_VALUE: Balance = 1;

        /// Positive case scenario:
        ///  - `call_runtime` is enabled
        ///  - the call is valid
        ///  - the call execution succeeds
        #[cfg(feature = "permissive-node")]
        #[ink_e2e::test]
        async fn transfer_with_call_runtime_works(
            mut client: Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let constructor = RuntimeCallerRef::new();
            let contract_acc_id = client
                .instantiate(
                    "call-runtime",
                    &ink_e2e::alice(),
                    constructor,
                    CONTRACT_BALANCE,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            let receiver: AccountId = default_accounts::<DefaultEnvironment>().bob;

            let contract_balance_before = client
                .balance(contract_acc_id)
                .await
                .expect("Failed to get account balance");
            let receiver_balance_before = client
                .balance(receiver)
                .await
                .expect("Failed to get account balance");

            // when
            let transfer_message = build_message::<RuntimeCallerRef>(contract_acc_id)
                .call(|caller| caller.transfer_through_runtime(receiver, TRANSFER_VALUE));

            let call_res = client
                .call(&ink_e2e::alice(), transfer_message, 0, None)
                .await
                .expect("call failed");

            assert!(call_res.return_value().is_ok());

            // then
            let contract_balance_after = client
                .balance(contract_acc_id)
                .await
                .expect("Failed to get account balance");
            let receiver_balance_after = client
                .balance(receiver)
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

        /// Negative case scenario:
        ///  - `call_runtime` is enabled
        ///  - the call is valid
        ///  - the call execution fails
        #[cfg(feature = "permissive-node")]
        #[ink_e2e::test]
        async fn transfer_with_call_runtime_fails_when_execution_fails(
            mut client: Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let constructor = RuntimeCallerRef::new();
            let contract_acc_id = client
                .instantiate(
                    "call-runtime",
                    &ink_e2e::alice(),
                    constructor,
                    CONTRACT_BALANCE,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            let receiver: AccountId = default_accounts::<DefaultEnvironment>().bob;

            // when
            let transfer_message = build_message::<RuntimeCallerRef>(contract_acc_id)
                .call(|caller| {
                    caller.transfer_through_runtime(receiver, INSUFFICIENT_TRANSFER_VALUE)
                });

            let call_res = client
                .call_dry_run(&ink_e2e::alice(), &transfer_message, 0, None)
                .await
                .return_value();

            // then
            assert!(matches!(call_res, Err(RuntimeError::CallRuntimeFailed)));

            Ok(())
        }

        /// Negative case scenario:
        ///  - `call_runtime` is enabled
        ///  - the call is invalid
        #[cfg(feature = "permissive-node")]
        #[ink_e2e::test]
        async fn transfer_with_call_runtime_fails_when_call_is_invalid(
            mut client: Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let constructor = RuntimeCallerRef::new();
            let contract_acc_id = client
                .instantiate(
                    "call-runtime",
                    &ink_e2e::alice(),
                    constructor,
                    CONTRACT_BALANCE,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // when
            let transfer_message = build_message::<RuntimeCallerRef>(contract_acc_id)
                .call(|caller| caller.call_nonexistent_extrinsic());

            let call_res = client
                .call_dry_run(&ink_e2e::alice(), &transfer_message, 0, None)
                .await;

            // then
            assert!(call_res.is_err());

            Ok(())
        }

        /// Negative case scenario:
        ///  - `call_runtime` is disabled
        #[cfg(not(feature = "permissive-node"))]
        #[ink_e2e::test]
        async fn call_runtime_fails_when_forbidden(
            mut client: Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let constructor = RuntimeCallerRef::new();
            let contract_acc_id = client
                .instantiate("call-runtime", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let receiver: AccountId = default_accounts::<DefaultEnvironment>().bob;

            let transfer_message = build_message::<RuntimeCallerRef>(contract_acc_id)
                .call(|caller| caller.transfer_through_runtime(receiver, TRANSFER_VALUE));

            // when
            let call_res = client
                .call(&ink_e2e::alice(), transfer_message, 0, None)
                .await;

            // then
            assert!(matches!(call_res, Err(ink_e2e::Error::CallExtrinsic(_))));

            Ok(())
        }
    }
}
