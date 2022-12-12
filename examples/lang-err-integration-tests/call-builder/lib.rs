//! # Integration Tests for `LangError`
//!
//! This contract is used to ensure that the behavior around `LangError`s works as expected.
//!
//! In particular, it exercises the codepaths that stem from the usage of the
//! [`CallBuilder`](`ink_env::call::CallBuilder`) and [`CreateBuilder`](`ink_env::call::CreateBuilder`)
//! structs.
//!
//! This differs from the codepath used by external tooling, such as `cargo-contract` or the
//! `Contracts-UI` which instead depend on methods from the Contracts pallet which are exposed via
//! RPC.
//!
//! Note that during testing we make use of ink!'s end-to-end testing features, so ensure that you
//! have a node which includes the Contracts pallet running alongside your tests.

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod call_builder {
    use ink::env::{
        call::{
            build_call,
            build_create,
            Call,
            ExecutionInput,
            Selector,
        },
        DefaultEnvironment,
    };

    #[ink(storage)]
    #[derive(Default)]
    pub struct CallBuilderTest {}

    impl CallBuilderTest {
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Call a contract using the `CallBuilder`.
        ///
        /// Since we can't use the `CallBuilder` in a test environment directly we need this
        /// wrapper to test things like crafting calls with invalid selectors.
        ///
        /// We also wrap the output in an `Option` since we can't return a `Result` directly from a
        /// contract message without erroring out ourselves.
        #[ink(message)]
        pub fn call(
            &mut self,
            address: AccountId,
            selector: [u8; 4],
        ) -> Option<ink::LangError> {
            let result = build_call::<DefaultEnvironment>()
                .call_type(Call::new().callee(address))
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<Result<(), ink::LangError>>()
                .fire()
                .expect("Error from the Contracts pallet.");

            match result {
                Ok(_) => None,
                Err(e @ ink::LangError::CouldNotReadInput) => Some(e),
                Err(_) => {
                    unimplemented!("No other `LangError` variants exist at the moment.")
                }
            }
        }

        /// Instantiate a contract using the `CreateBuilder`.
        ///
        /// Since we can't use the `CreateBuilder` in a test environment directly we need this
        /// wrapper to test things like crafting calls with invalid selectors.
        ///
        /// We also wrap the output in an `Option` since we can't return a `Result` directly from a
        /// contract message without erroring out ourselves.
        #[ink(message)]
        pub fn call_instantiate(
            &mut self,
            code_hash: Hash,
            selector: [u8; 4],
            init_value: bool,
        ) -> Option<ink::LangError> {
            let result = build_create::<DefaultEnvironment>()
                .code_hash(code_hash)
                .gas_limit(0)
                .endowment(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(selector)).push_arg(init_value),
                )
                .salt_bytes(&[0xDE, 0xAD, 0xBE, 0xEF])
                .returns::<constructors_return_value::ConstructorsReturnValueRef>()
                .params()
                .try_instantiate()
                .expect("Error from the Contracts pallet.");

            match result {
                Ok(_) => None,
                Err(e @ ink::LangError::CouldNotReadInput) => Some(e),
                Err(_) => {
                    unimplemented!("No other `LangError` variants exist at the moment.")
                }
            }
        }

        /// Attempt to instantiate a contract using the `CreateBuilder`.
        ///
        /// Since we can't use the `CreateBuilder` in a test environment directly we need this
        /// wrapper to test things like crafting calls with invalid selectors.
        ///
        /// We also wrap the output in an `Option` since we can't return a `Result` directly from a
        /// contract message without erroring out ourselves.
        #[ink(message)]
        pub fn call_instantiate_fallible(
            &mut self,
            code_hash: Hash,
            selector: [u8; 4],
            init_value: bool,
        ) -> Option<
            Result<
                Result<AccountId, constructors_return_value::ConstructorError>,
                ink::LangError,
            >,
        > {
            let lang_result = build_create::<DefaultEnvironment>()
                .code_hash(code_hash)
                .gas_limit(0)
                .endowment(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(selector)).push_arg(init_value),
                )
                .salt_bytes(&[0xDE, 0xAD, 0xBE, 0xEF])
                .returns::<Result<
                    constructors_return_value::ConstructorsReturnValueRef,
                    constructors_return_value::ConstructorError,
                >>()
                .params()
                .try_instantiate_fallible()
                .expect("Error from the Contracts pallet.");

            Some(lang_result.map(|contract_result| {
                contract_result.map(|inner| ink::ToAccountId::to_account_id(&inner))
            }))
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::CallBuilderTestRef;
        use ink_e2e::build_message;
        use integration_flipper::FlipperRef;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(
            additional_contracts = "../integration-flipper/Cargo.toml ../constructors-return-value/Cargo.toml"
        )]
        async fn e2e_invalid_message_selector_can_be_handled(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = CallBuilderTestRef::new();
            let contract_acc_id = client
                .instantiate("call_builder", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let flipper_constructor = FlipperRef::default();
            let flipper_acc_id = client
                .instantiate(
                    "integration_flipper",
                    &ink_e2e::alice(),
                    flipper_constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate `flipper` failed")
                .account_id;

            let flipper_get = build_message::<FlipperRef>(flipper_acc_id)
                .call(|contract| contract.get());
            let get_call_result = client
                .call(&ink_e2e::alice(), flipper_get, 0, None)
                .await
                .expect("Calling `flipper::get` failed");
            let initial_value = get_call_result
                .value
                .expect("Input is valid, call must not fail.");

            let selector = ink::selector_bytes!("invalid_selector");
            let call = build_message::<CallBuilderTestRef>(contract_acc_id)
                .call(|contract| contract.call(flipper_acc_id, selector));
            let call_result = client
                .call(&ink_e2e::alice(), call, 0, None)
                .await
                .expect("Calling `call_builder::call` failed");

            let flipper_result = call_result
                .value
                .expect("Call to `call_builder::call` failed");

            assert!(matches!(
                flipper_result,
                Some(ink::LangError::CouldNotReadInput)
            ));

            let flipper_get = build_message::<FlipperRef>(flipper_acc_id)
                .call(|contract| contract.get());
            let get_call_result = client
                .call(&ink_e2e::alice(), flipper_get, 0, None)
                .await
                .expect("Calling `flipper::get` failed");
            let flipped_value = get_call_result
                .value
                .expect("Input is valid, call must not fail.");
            assert!(flipped_value == initial_value);

            Ok(())
        }

        #[ink_e2e::test(additional_contracts = "../constructors-return-value/Cargo.toml")]
        async fn e2e_create_builder_works_with_valid_selector(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = CallBuilderTestRef::new();
            let contract_acc_id = client
                .instantiate("call_builder", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload("constructors_return_value", &ink_e2e::bob(), None)
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("new");
            let init_value = true;
            let call =
                build_message::<CallBuilderTestRef>(contract_acc_id).call(|contract| {
                    contract.call_instantiate(code_hash, selector, init_value)
                });
            let call_result = client
                .call(&ink_e2e::bob(), call, 0, None)
                .await
                .expect("Calling `call_builder::call_instantiate` failed")
                .value
                .expect("Dispatching `call_builder::call_instantiate` failed.");

            assert!(
                call_result.is_none(),
                "Call using valid selector failed, when it should've succeeded."
            );

            Ok(())
        }

        #[ink_e2e::test(additional_contracts = "../constructors-return-value/Cargo.toml")]
        async fn e2e_create_builder_fails_with_invalid_selector(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = CallBuilderTestRef::new();
            let contract_acc_id = client
                .instantiate("call_builder", &ink_e2e::charlie(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload("constructors_return_value", &ink_e2e::charlie(), None)
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("invalid_selector");
            let init_value = true;
            let call =
                build_message::<CallBuilderTestRef>(contract_acc_id).call(|contract| {
                    contract.call_instantiate(code_hash, selector, init_value)
                });
            let call_result = client
                .call(&ink_e2e::charlie(), call, 0, None)
                .await
                .expect("Client failed to call `call_builder::call_instantiate`.")
                .value
                .expect("Dispatching `call_builder::call_instantiate` failed.");

            assert!(
                matches!(call_result, Some(ink::LangError::CouldNotReadInput)),
                "Call using invalid selector succeeded, when it should've failed."
            );

            Ok(())
        }

        #[ink_e2e::test(additional_contracts = "../constructors-return-value/Cargo.toml")]
        async fn e2e_create_builder_with_infallible_revert_constructor_encodes_ok(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = CallBuilderTestRef::new();
            let contract_acc_id = client
                .instantiate("call_builder", &ink_e2e::dave(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload("constructors_return_value", &ink_e2e::dave(), None)
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("revert_new");
            let init_value = false;
            let call =
                build_message::<CallBuilderTestRef>(contract_acc_id).call(|contract| {
                    contract.call_instantiate(code_hash, selector, init_value)
                });
            let call_result = client.call(&mut ink_e2e::dave(), call, 0, None).await;

            assert!(
                call_result.is_err(),
                "Call execution should've failed, but didn't."
            );
            let contains_err_msg = match call_result.unwrap_err() {
                ink_e2e::Error::CallDryRun(dry_run) => {
                    String::from_utf8_lossy(&dry_run.debug_message)
                        .contains("The callee reverted, but did not encode an error in the output buffer.")
                }
                _ => false,
            };
            assert!(
                contains_err_msg,
                "Call execution failed for an unexpected reason."
            );

            Ok(())
        }

        #[ink_e2e::test(additional_contracts = "../constructors-return-value/Cargo.toml")]
        async fn e2e_create_builder_can_handle_fallible_constructor_success(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = CallBuilderTestRef::new();
            let contract_acc_id = client
                .instantiate("call_builder", &ink_e2e::eve(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload("constructors_return_value", &ink_e2e::eve(), None)
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("try_new");
            let init_value = true;
            let call =
                build_message::<CallBuilderTestRef>(contract_acc_id).call(|contract| {
                    contract.call_instantiate_fallible(code_hash, selector, init_value)
                });
            let call_result = client
                .call(&mut ink_e2e::eve(), call, 0, None)
                .await
                .expect("Calling `call_builder::call_instantiate_fallible` failed")
                .value
                .expect("Dispatching `call_builder::call_instantiate_fallible` failed.");

            assert!(
                call_result.unwrap().is_ok(),
                "Call to falliable constructor failed, when it should have succeeded."
            );

            Ok(())
        }

        #[ink_e2e::test(additional_contracts = "../constructors-return-value/Cargo.toml")]
        async fn e2e_create_builder_can_handle_fallible_constructor_error(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = CallBuilderTestRef::new();
            let contract_acc_id = client
                .instantiate("call_builder", &ink_e2e::ferdie(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload("constructors_return_value", &ink_e2e::ferdie(), None)
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("try_new");
            let init_value = false;
            let call =
                build_message::<CallBuilderTestRef>(contract_acc_id).call(|contract| {
                    contract.call_instantiate_fallible(code_hash, selector, init_value)
                });
            let call_result = client
                .call(&mut ink_e2e::ferdie(), call, 0, None)
                .await
                .expect("Calling `call_builder::call_instantiate_fallible` failed")
                .value
                .expect("Dispatching `call_builder::call_instantiate_fallible` failed.");

            let contract_result = call_result
                .unwrap()
                .expect("Dispatching `constructors_return_value::try_new` failed.");

            assert!(
                matches!(
                    contract_result,
                    Err(constructors_return_value::ConstructorError)
                ),
                "Got an unexpected error from the contract."
            );

            Ok(())
        }

        #[ink_e2e::test(additional_contracts = "../constructors-return-value/Cargo.toml")]
        async fn e2e_create_builder_with_fallible_revert_constructor_encodes_ok(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = CallBuilderTestRef::new();
            let contract_acc_id = client
                .instantiate("call_builder", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload("constructors_return_value", &ink_e2e::alice(), None)
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("try_revert_new");
            let init_value = true;
            let call =
                build_message::<CallBuilderTestRef>(contract_acc_id).call(|contract| {
                    contract.call_instantiate_fallible(code_hash, selector, init_value)
                });
            let call_result = client.call(&mut ink_e2e::alice(), call, 0, None).await;

            assert!(
                call_result.is_err(),
                "Call execution should've failed, but didn't."
            );

            let contains_err_msg = match call_result.unwrap_err() {
                ink_e2e::Error::CallDryRun(dry_run) => {
                    String::from_utf8_lossy(&dry_run.debug_message)
                        .contains("The callee reverted, but did not encode an error in the output buffer.")
                }
                _ => false,
            };
            assert!(
                contains_err_msg,
                "Call execution failed for an unexpected reason."
            );

            Ok(())
        }

        #[ink_e2e::test(additional_contracts = "../constructors-return-value/Cargo.toml")]
        async fn e2e_create_builder_with_fallible_revert_constructor_encodes_err(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = CallBuilderTestRef::new();
            let contract_acc_id = client
                .instantiate("call_builder", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload("constructors_return_value", &ink_e2e::bob(), None)
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("try_revert_new");
            let init_value = false;
            let call =
                build_message::<CallBuilderTestRef>(contract_acc_id).call(|contract| {
                    contract.call_instantiate_fallible(code_hash, selector, init_value)
                });
            let call_result = client
                .call(&mut ink_e2e::bob(), call, 0, None)
                .await
                .expect(
                    "Client failed to call `call_builder::call_instantiate_fallible`.",
                )
                .value
                .expect("Dispatching `call_builder::call_instantiate_fallible` failed.");

            assert!(
                matches!(call_result, Some(Err(ink::LangError::CouldNotReadInput))),
                "The callee manually encoded `CouldNotReadInput` to the output buffer, we should've
                 gotten that back."
            );

            Ok(())
        }
    }
}
