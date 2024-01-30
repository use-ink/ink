#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::primitives::AccountId;
use sp_runtime::MultiAddress;

/// A part of the runtime dispatchable API.
///
/// For now, `ink!` doesn't provide any support for exposing the real `RuntimeCall` enum,
/// which fully describes the composed API of all the pallets present in runtime. Hence,
/// in order to use `call-runtime` functionality, we have to provide at least a partial
/// object, which correctly encodes the target extrinsic.
///
/// You can investigate the full `RuntimeCall` definition by either expanding
/// `construct_runtime!` macro application or by using secondary tools for reading chain
/// metadata, like `subxt`.
#[ink::scale_derive(Encode)]
enum RuntimeCall {
    /// This index can be found by investigating runtime configuration. You can check the
    /// pallet order inside `construct_runtime!` block and read the position of your
    /// pallet (0-based).
    ///
    ///
    /// [See here for more.](https://substrate.stackexchange.com/questions/778/how-to-get-pallet-index-u8-of-a-pallet-in-runtime)
    #[codec(index = 4)]
    Balances(BalancesCall),
}

#[ink::scale_derive(Encode)]
enum BalancesCall {
    /// This index can be found by investigating the pallet dispatchable API. In your
    /// pallet code, look for `#[pallet::call]` section and check
    /// `#[pallet::call_index(x)]` attribute of the call. If these attributes are
    /// missing, use source-code order (0-based).
    #[codec(index = 0)]
    Transfer {
        dest: MultiAddress<AccountId, ()>,
        #[codec(compact)]
        value: u128,
    },
}

#[ink::contract]
mod runtime_call {
    use crate::{
        BalancesCall,
        RuntimeCall,
    };

    use ink::env::Error as EnvError;

    /// A trivial contract with a single message, that uses `call-runtime` API for
    /// performing native token transfer.
    #[ink(storage)]
    #[derive(Default)]
    pub struct RuntimeCaller;

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum RuntimeError {
        CallRuntimeFailed,
    }

    impl From<EnvError> for RuntimeError {
        fn from(e: EnvError) -> Self {
            use ink::env::ReturnErrorCode;
            match e {
                EnvError::ReturnError(ReturnErrorCode::CallRuntimeFailed) => {
                    RuntimeError::CallRuntimeFailed
                }
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
            self.env()
                .call_runtime(&RuntimeCall::Balances(BalancesCall::Transfer {
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
        use ink_e2e::{
            ChainBackend,
            ContractsBackend,
        };

        use ink::{
            env::{
                test::default_accounts,
                DefaultEnvironment,
            },
            primitives::AccountId,
        };

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        /// The base number of indivisible units for balances on the
        /// `substrate-contracts-node`.
        const UNIT: Balance = 1_000_000_000_000;

        /// The contract will be given 1000 tokens during instantiation.
        const CONTRACT_BALANCE: Balance = 1_000 * UNIT;

        /// The receiver will get enough funds to have the required existential deposit.
        ///
        /// If your chain has this threshold higher, increase the transfer value.
        const TRANSFER_VALUE: Balance = 1 / 10 * UNIT;

        /// An amount that is below the existential deposit, so that a transfer to an
        /// empty account fails.
        ///
        /// Must not be zero, because such an operation would be a successful no-op.
        const INSUFFICIENT_TRANSFER_VALUE: Balance = 1;

        /// Positive case scenario:
        ///  - the call is valid
        ///  - the call execution succeeds
        #[ink_e2e::test]
        async fn transfer_with_call_runtime_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = RuntimeCallerRef::new();
            let contract = client
                .instantiate("call-runtime", &ink_e2e::alice(), &mut constructor)
                .value(CONTRACT_BALANCE)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<RuntimeCaller>();

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
                call_builder.transfer_through_runtime(receiver, TRANSFER_VALUE);

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

        /// Negative case scenario:
        ///  - the call is valid
        ///  - the call execution fails
        #[ink_e2e::test]
        async fn transfer_with_call_runtime_fails_when_execution_fails<
            Client: E2EBackend,
        >(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = RuntimeCallerRef::new();
            let contract = client
                .instantiate("call-runtime", &ink_e2e::alice(), &mut constructor)
                .value(CONTRACT_BALANCE)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<RuntimeCaller>();

            let receiver: AccountId = default_accounts::<DefaultEnvironment>().bob;

            // when
            let transfer_message = call_builder
                .transfer_through_runtime(receiver, INSUFFICIENT_TRANSFER_VALUE);

            let call_res = client
                .call(&ink_e2e::alice(), &transfer_message)
                .dry_run()
                .await?
                .return_value();

            // then
            assert!(matches!(call_res, Err(RuntimeError::CallRuntimeFailed)));

            Ok(())
        }

        /// Negative case scenario:
        ///  - the call is invalid
        #[ink_e2e::test]
        async fn transfer_with_call_runtime_fails_when_call_is_invalid<
            Client: E2EBackend,
        >(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = RuntimeCallerRef::new();
            let contract = client
                .instantiate("call-runtime", &ink_e2e::alice(), &mut constructor)
                .value(CONTRACT_BALANCE)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<RuntimeCaller>();

            // when
            let transfer_message = call_builder.call_nonexistent_extrinsic();

            let call_res = client
                .call(&ink_e2e::alice(), &transfer_message)
                .dry_run()
                .await;

            // then
            assert!(call_res.is_err());

            Ok(())
        }
    }
}
