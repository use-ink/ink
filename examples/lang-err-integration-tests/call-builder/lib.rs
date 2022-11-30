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
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(
            additional_contracts = "../integration-flipper/Cargo.toml ../constructors-return-value/Cargo.toml"
        )]
        async fn e2e_invalid_message_selector_can_be_handled(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            use call_builder::contract_types::ink_primitives::LangError as E2ELangError;

            let constructor = call_builder::constructors::new();
            let contract_acc_id = client
                .instantiate(&mut ink_e2e::charlie(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let flipper_constructor = integration_flipper::constructors::default();
            let flipper_acc_id = client
                .instantiate(&mut ink_e2e::charlie(), flipper_constructor, 0, None)
                .await
                .expect("instantiate `flipper` failed")
                .account_id;

            let get_call_result = client
                .call(
                    &mut ink_e2e::charlie(),
                    flipper_acc_id.clone(),
                    integration_flipper::messages::get(),
                    0,
                    None,
                )
                .await
                .expect("Calling `flipper::get` failed");
            let initial_value = get_call_result
                .value
                .expect("Input is valid, call must not fail.");

            let flipper_ink_acc_id =
                ink::primitives::AccountId::try_from(flipper_acc_id.clone().as_ref())
                    .unwrap();
            let invalid_selector = [0x00, 0x00, 0x00, 0x00];
            let call_result = client
                .call(
                    &mut ink_e2e::charlie(),
                    contract_acc_id.clone(),
                    call_builder::messages::call(flipper_ink_acc_id, invalid_selector),
                    0,
                    None,
                )
                .await
                .expect("Calling `call_builder::call` failed");

            let flipper_result = call_result
                .value
                .expect("Call to `call_builder::call` failed");

            assert!(matches!(
                flipper_result,
                Some(E2ELangError::CouldNotReadInput)
            ));

            let get_call_result = client
                .call(
                    &mut ink_e2e::charlie(),
                    flipper_acc_id.clone(),
                    integration_flipper::messages::get(),
                    0,
                    None,
                )
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

            let new_selector = [0x9B, 0xAE, 0x9D, 0x5E];
            let call_result = client
                .call(
                    &mut ink_e2e::dave(),
                    contract_acc_id.clone(),
                    call_builder::messages::call_instantiate(
                        ink_e2e::utils::runtime_hash_to_ink_hash::<
                            ink::env::DefaultEnvironment,
                        >(&code_hash),
                        new_selector,
                        true,
                    ),
                    0,
                    None,
                )
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
            use call_builder::contract_types::ink_primitives::LangError as E2ELangError;

            let constructor = call_builder::constructors::new();
            let contract_acc_id = client
                .instantiate(&mut ink_e2e::eve(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let code_hash = client
                .upload(
                    &mut ink_e2e::eve(),
                    constructors_return_value::CONTRACT_PATH,
                    None,
                )
                .await
                .expect("upload `constructors_return_value` failed")
                .code_hash;

            let invalid_selector = [0x00, 0x00, 0x00, 0x00];
            let call_result = client
                .call(
                    &mut ink_e2e::eve(),
                    contract_acc_id.clone(),
                    call_builder::messages::call_instantiate(
                        ink_e2e::utils::runtime_hash_to_ink_hash::<
                            ink::env::DefaultEnvironment,
                        >(&code_hash),
                        invalid_selector,
                        true,
                    ),
                    0,
                    None,
                )
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
        async fn e2e_create_builder_with_revert_constructor(
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
    }
}
