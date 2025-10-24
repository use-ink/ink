//! # Integration Tests for `LangError`
//!
//! This contract is used to ensure that the behavior around `LangError`s works as
//! expected.
//!
//! In particular, it exercises the codepaths that stem from the usage of the
//! [`CallBuilder`](`ink::env::call::CallBuilder`) and
//! [`CreateBuilder`](`ink::env::call::CreateBuilder`) structs.
//!
//! This differs from the codepath used by external tooling, such as `cargo-contract` or
//! the `Contracts-UI` which instead depend on methods from `pallet-revive` which are
//! exposed via RPC.
//!
//! Note that during testing we make use of ink!'s end-to-end testing features, so ensure
//! that you have a node which includes `pallet-revive` running alongside your tests.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod call_builder {
    use ink::env::{
        DefaultEnvironment,
        call::{
            ExecutionInput,
            Selector,
            build_call,
        },
    };

    #[ink(storage)]
    #[derive(Default)]
    pub struct CallBuilderDelegateTest {
        /// Since we're going to `DelegateCall` into the `incrementer` contract, we need
        /// to make sure our storage layout matches.
        value: i32,
    }

    impl CallBuilderDelegateTest {
        #[ink(constructor)]
        pub fn new(value: i32) -> Self {
            Self { value }
        }

        /// Call a contract using the `CallBuilder`.
        ///
        /// Since we can't use the `CallBuilder` in a test environment directly we need
        /// this wrapper to test things like crafting calls with invalid
        /// selectors.
        ///
        /// We also wrap the output in an `Option` since we can't return a `Result`
        /// directly from a contract message without erroring out ourselves.
        #[ink(message)]
        pub fn delegate(
            &mut self,
            address: Address,
            selector: [u8; 4],
        ) -> Option<ink::LangError> {
            let result = build_call::<DefaultEnvironment>()
                .delegate(address)
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<bool>()
                .try_invoke()
                .expect("Error from `pallet-revive`.");

            match result {
                Ok(_) => None,
                Err(e @ ink::LangError::CouldNotReadInput) => Some(e),
                Err(_) => {
                    unimplemented!("No other `LangError` variants exist at the moment.")
                }
            }
        }

        /// Call a contract using the `CallBuilder`.
        ///
        /// Since we can't use the `CallBuilder` in a test environment directly we need
        /// this wrapper to test things like crafting calls with invalid
        /// selectors.
        ///
        /// This message does not allow the caller to handle any `LangErrors`, for that
        /// use the `call` message instead.
        #[ink(message)]
        pub fn invoke(&mut self, address: Address, selector: [u8; 4]) -> i32 {
            use ink::env::call::build_call;

            build_call::<DefaultEnvironment>()
                .delegate(address)
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<i32>()
                .invoke()
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::{
            ChainBackend,
            ContractsBackend,
        };

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_invalid_message_selector_can_be_handled<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::bob(), 10_000_000_000_000)
                .await;

            let mut constructor = CallBuilderDelegateTestRef::new(Default::default());
            let contract = client
                .instantiate("call_builder_delegate", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<CallBuilderDelegateTest>();

            let mut incrementer_constructor = incrementer::IncrementerRef::new(42);
            let address = client
                .instantiate("incrementer", &origin, &mut incrementer_constructor)
                .submit()
                .await
                .expect("instantiating `incrementer` failed")
                .addr;

            let selector = ink::selector_bytes!("invalid_selector");
            let call = call_builder.delegate(address, selector);
            let call_result = client
                .call(&origin, &call)
                .submit()
                .await
                .expect("Calling `call_builder::delegate` failed");

            assert!(matches!(
                call_result.return_value(),
                Some(ink::LangError::CouldNotReadInput)
            ));

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_invalid_message_selector_panics_on_invoke<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::charlie(), 10_000_000_000_000)
                .await;

            let mut constructor = CallBuilderDelegateTestRef::new(Default::default());
            let contract = client
                .instantiate("call_builder_delegate", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiating `call_builder_delegate` failed");
            let mut call_builder = contract.call_builder::<CallBuilderDelegateTest>();

            let mut incrementer_constructor = incrementer::IncrementerRef::new(42);
            let address = client
                .instantiate("incrementer", &origin, &mut incrementer_constructor)
                .submit()
                .await
                .expect("instantiating `incrementer` failed")
                .addr;

            // Since `LangError`s can't be handled by the `CallBuilder::invoke()` method
            // we expect this to panic.
            let selector = ink::selector_bytes!("invalid_selector");
            let call = call_builder.invoke(address, selector);
            let call_result = client.call(&origin, &call).dry_run().await?;
            assert!(call_result.did_revert());
            let err_msg = String::from_utf8_lossy(call_result.return_data());
            assert!(
                err_msg.contains("Cross-contract call failed with CouldNotReadInput")
            );

            Ok(())
        }
    }
}
