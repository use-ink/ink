//! # Integration Tests for `LangError`
//!
//! This contract is used to ensure that the behavior around `LangError`s works as expected.
//!
//! It makes use of ink!'s end-to-end testing features, so ensure that you have a node which
//! includes the Contract's pallet running alongside your tests.

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod call_builder {
    use ink::env::{
        call::{
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
        #[ink(message)]
        pub fn call(
            &mut self,
            address: AccountId,
            selector: [u8; 4],
        ) -> Option<::ink::LangError> {
            use ink::env::call::build_call;

            let result = build_call::<DefaultEnvironment>()
                .call_type(Call::new().callee(address))
                .exec_input(ExecutionInput::new(Selector::new(selector)))
                .returns::<Result<(), ::ink::LangError>>()
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

        #[ink(message)]
        pub fn call_instantiate(
            &mut self,
            code_hash: Hash,
            selector: [u8; 4],
            init_value: bool,
        ) -> Option<::ink::LangError> {
            use ink::env::call::build_create;

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
            ::ink::env::debug_println!("Result from `instantiate` {:?}", &result);

            match result {
                Ok(_) => None,
                Err(e @ ink::LangError::CouldNotReadInput) => Some(e),
                Err(_) => {
                    unimplemented!("No other `LangError` variants exist at the moment.")
                }
            }
        }

        #[ink(message)]
        pub fn call_instantiate_with_result(
            &mut self,
            code_hash: Hash,
            selector: [u8; 4],
            init_value: bool,
        ) -> Option<Result<AccountId, constructors_return_value::ConstructorError>>
        {
            use ink::env::call::build_create;

            let result = build_create::<DefaultEnvironment>()
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
                .try_instantiate_with_result()
                .expect("Error from the Contracts pallet.")
                .expect("Dispatch should have succeeded.");
            ::ink::env::debug_println!("Result from `instantiate` {:?}", &result);

            Some(result)
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
                .instantiate("call_builder", &ink_e2e::charlie(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let flipper_constructor = FlipperRef::default();
            let flipper_acc_id = client
                .instantiate(
                    "integration_flipper",
                    &ink_e2e::charlie(),
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
                .call(&ink_e2e::charlie(), flipper_get, 0, None)
                .await
                .expect("Calling `flipper::get` failed");
            let initial_value = get_call_result
                .value
                .expect("Input is valid, call must not fail.");

            let invalid_selector = [0x00, 0x00, 0x00, 0x00];
            let call = build_message::<CallBuilderTestRef>(contract_acc_id)
                .call(|contract| contract.call(flipper_acc_id, invalid_selector));
            let call_result = client
                .call(&ink_e2e::charlie(), call, 0, None)
                .await
                .expect("Calling `call_builder::call` failed");

            let flipper_result = call_result
                .value
                .expect("Call to `call_builder::call` failed");

            assert!(matches!(
                flipper_result,
                Some(::ink::LangError::CouldNotReadInput)
            ));

            let flipper_get = build_message::<FlipperRef>(flipper_acc_id)
                .call(|contract| contract.get());
            let get_call_result = client
                .call(&ink_e2e::charlie(), flipper_get, 0, None)
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
                .instantiate("call_builder", &ink_e2e::dave(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload("constructors_return_value", &ink_e2e::dave(), None)
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let new_selector = [0x9B, 0xAE, 0x9D, 0x5E];
            let call =
                build_message::<CallBuilderTestRef>(contract_acc_id).call(|contract| {
                    contract.call_instantiate(code_hash, new_selector, true)
                });
            let call_result = client
                .call(&ink_e2e::dave(), call, 0, None)
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
                .instantiate("call_builder", &ink_e2e::eve(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload("constructors_return_value", &ink_e2e::eve(), None)
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let invalid_selector = [0x00, 0x00, 0x00, 0x00];
            let call =
                build_message::<CallBuilderTestRef>(contract_acc_id).call(|contract| {
                    contract.call_instantiate(code_hash, invalid_selector, true)
                });
            let call_result = client
                .call(&ink_e2e::eve(), call, 0, None)
                .await
                .expect("Client failed to call `call_builder::call_instantiate`.")
                .value
                .expect("Dispatching `call_builder::call_instantiate` failed.");

            assert!(
                matches!(call_result, Some(E2ELangError::CouldNotReadInput)),
                "Call using invalid selector succeeded, when it should've failed."
            );

            Ok(())
        }

        #[ink_e2e::test(additional_contracts = "../constructors-return-value/Cargo.toml")]
        async fn e2e_create_builder_with_infallible_revert_constructor_encodes_ok(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = call_builder::constructors::new();
            let contract_acc_id = client
                .instantiate(&mut ink_e2e::ferdie(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload(
                    &mut ink_e2e::ferdie(),
                    constructors_return_value::CONTRACT_PATH,
                    None,
                )
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let revert_new_selector = [0x90, 0xC9, 0xFE, 0x94];
            let call_result = client
                .call(
                    &mut ink_e2e::ferdie(),
                    contract_acc_id.clone(),
                    call_builder::messages::call_instantiate(
                        ink_e2e::utils::runtime_hash_to_ink_hash::<
                            ink::env::DefaultEnvironment,
                        >(&code_hash),
                        revert_new_selector,
                        false,
                    ),
                    0,
                    None,
                )
                .await;

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
            let constructor = call_builder::constructors::new();
            let contract_acc_id = client
                .instantiate(&mut ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload(
                    &mut ink_e2e::alice(),
                    constructors_return_value::CONTRACT_PATH,
                    None,
                )
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("try_new");
            let success = true;
            let call_result = dbg!(
                client
                    .call(
                        &mut ink_e2e::alice(),
                        contract_acc_id.clone(),
                        call_builder::messages::call_instantiate_with_result(
                            ink_e2e::utils::runtime_hash_to_ink_hash::<
                                ink::env::DefaultEnvironment,
                            >(&code_hash),
                            selector,
                            success,
                        ),
                        0,
                        None,
                    )
                    .await
                    .expect("Calling `call_builder::call_instantiate` failed")
                    .value
            )
            .expect("Dispatching `call_builder::call_instantiate` failed.");

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
            let constructor = call_builder::constructors::new();
            let contract_acc_id = client
                .instantiate(&mut ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload(
                    &mut ink_e2e::bob(),
                    constructors_return_value::CONTRACT_PATH,
                    None,
                )
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("try_new");
            let success = false;
            let call_result = dbg!(
                client
                    .call(
                        &mut ink_e2e::bob(),
                        contract_acc_id.clone(),
                        call_builder::messages::call_instantiate_with_result(
                            ink_e2e::utils::runtime_hash_to_ink_hash::<
                                ink::env::DefaultEnvironment,
                            >(&code_hash),
                            selector,
                            success,
                        ),
                        0,
                        None,
                    )
                    .await
                    .expect("Calling `call_builder::call_instantiate` failed")
                    .value
            )
            .expect("Dispatching `call_builder::call_instantiate` failed.");

            assert!(
                call_result.unwrap().is_err(),
                "Call to falliable constructor succeeded, when it should have failed."
            );

            Ok(())
        }

        #[ink_e2e::test(additional_contracts = "../constructors-return-value/Cargo.toml")]
        async fn e2e_create_builder_with_fallible_revert_constructor_encodes_ok(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = call_builder::constructors::new();
            let contract_acc_id = client
                .instantiate(&mut ink_e2e::charlie(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload(
                    &mut ink_e2e::charlie(),
                    constructors_return_value::CONTRACT_PATH,
                    None,
                )
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("try_revert_new");
            let init_value = true;
            let call_result = client
                .call(
                    &mut ink_e2e::charlie(),
                    contract_acc_id.clone(),
                    call_builder::messages::call_instantiate_with_result(
                        ink_e2e::utils::runtime_hash_to_ink_hash::<
                            ink::env::DefaultEnvironment,
                        >(&code_hash),
                        selector,
                        init_value,
                    ),
                    0,
                    None,
                )
                .await;

            assert!(
                call_result.is_err(),
                "Call execution should've failed, but didn't."
            );

            let contains_err_msg = match call_result.unwrap_err() {
                ink_e2e::Error::CallDryRun(dry_run) => {
                    String::from_utf8_lossy(&dry_run.debug_message)
                        .contains(
                            "Since the contract reverted, we only expect an `Error` from the constructor. \
                             Otherwise we would be in the `AccountId` branch."
                        )
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
            let constructor = call_builder::constructors::new();
            let contract_acc_id = client
                .instantiate(&mut ink_e2e::dave(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload(
                    &mut ink_e2e::dave(),
                    constructors_return_value::CONTRACT_PATH,
                    None,
                )
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let selector = ink::selector_bytes!("try_revert_new");
            let init_value = false;
            let call_result = client
                .call(
                    &mut ink_e2e::dave(),
                    contract_acc_id.clone(),
                    call_builder::messages::call_instantiate_with_result(
                        ink_e2e::utils::runtime_hash_to_ink_hash::<
                            ink::env::DefaultEnvironment,
                        >(&code_hash),
                        selector,
                        init_value,
                    ),
                    0,
                    None,
                )
                .await;

            assert!(
                call_result.is_err(),
                "Call execution should've failed, but didn't."
            );

            let contains_err_msg = match call_result.unwrap_err() {
                ink_e2e::Error::CallDryRun(dry_run) => {
                    String::from_utf8_lossy(&dry_run.debug_message).contains(
                        "If dispatch had failed, we shouldn't have been able to decode \
                         the nested `Result`.",
                    )
                }
                _ => false,
            };
            assert!(
                contains_err_msg,
                "Call execution failed for an unexpected reason."
            );

            Ok(())
        }
    }
}
