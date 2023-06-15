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
//! the `Contracts-UI` which instead depend on methods from the Contracts pallet which are
//! exposed via RPC.
//!
//! Note that during testing we make use of ink!'s end-to-end testing features, so ensure
//! that you have a node which includes the Contracts pallet running alongside your tests.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod call_builder {
    use ink::{
        env::{
            call::{
                build_call,
                ExecutionInput,
                Selector,
            },
            DefaultEnvironment,
        },
        prelude::{
            format,
            string::{
                String,
                ToString,
            },
        },
    };

    #[ink(storage)]
    #[derive(Default)]
    pub struct CallBuilderReturnValue {
        /// Since we're going to `DelegateCall` into the `incrementer` contract, we need
        /// to make sure our storage layout matches.
        value: i32,
    }

    impl CallBuilderReturnValue {
        #[ink(constructor)]
        pub fn new(value: i32) -> Self {
            Self { value }
        }

        /// Delegate a call to the given contract/selector and return the result.
        #[ink(message)]
        pub fn delegate_call(&mut self, code_hash: Hash, selector: [u8; 4]) -> i32 {
            use ink::env::call::build_call;

            build_call::<DefaultEnvironment>()
                .delegate(code_hash)
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<i32>()
                .invoke()
        }

        /// Delegate call to the given contract/selector and attempt to decode the return
        /// value into an `i8`.
        #[ink(message)]
        pub fn delegate_call_short_return_type(
            &mut self,
            code_hash: Hash,
            selector: [u8; 4],
        ) -> Result<i8, String> {
            use ink::env::call::build_call;

            let result = build_call::<DefaultEnvironment>()
                .delegate(code_hash)
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<i8>()
                .try_invoke();

            match result {
                Ok(Ok(value)) => Ok(value),
                Ok(Err(err)) => Err(format!("LangError: {:?}", err)),
                Err(ink::env::Error::Decode(_)) => Err("Decode Error".to_string()),
                Err(err) => Err(format!("Env Error: {:?}", err)),
            }
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_call_builder_return_value_returns_correct_value(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::alice(), 10_000_000_000_000)
                .await;

            let expected_value = 42;
            let constructor = CallBuilderReturnValueRef::new(expected_value);
            let call_builder = client
                .instantiate("call_builder_return_value", &origin, constructor, 0, None)
                .await
                .expect("instantiate failed");
            let mut call_builder_call = call_builder.call::<CallBuilderReturnValue>();

            let code_hash = client
                .upload("incrementer", &origin, None)
                .await
                .expect("upload `incrementer` failed")
                .code_hash;

            let selector = ink::selector_bytes!("get");
            let call = call_builder_call.delegate_call(code_hash, selector);
            let call_result = client
                .call(&origin, &call, 0, None)
                .await
                .expect("Client failed to call `call_builder::invoke`.")
                .return_value();

            assert_eq!(
                call_result, expected_value,
                "Decoded an unexpected value from the call."
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_call_builder_return_value_errors_if_return_data_too_long(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::alice(), 10_000_000_000_000)
                .await;

            let constructor = CallBuilderReturnValueRef::new(42);
            let call_builder = client
                .instantiate("call_builder_return_value", &origin, constructor, 0, None)
                .await
                .expect("instantiate failed");
            let mut call_builder_call = call_builder.call::<CallBuilderReturnValue>();

            let code_hash = client
                .upload("incrementer", &origin, None)
                .await
                .expect("upload `incrementer` failed")
                .code_hash;

            let selector = ink::selector_bytes!("get");
            let call = call_builder_call.delegate_call_short_return_type(code_hash, selector);
            let call_result: Result<i8, String> = client
                .call_dry_run(&origin, &call, 0, None)
                .await
                .return_value();

            assert!(
                call_result.is_err(),
                "Should fail of decoding an `i32` into an `i8`"
            );
            assert_eq!(
                "Decode Error".to_string(),
                call_result.unwrap_err(),
                "Should fail to decode short type if bytes remain from return data."
            );

            Ok(())
        }
    }
}
