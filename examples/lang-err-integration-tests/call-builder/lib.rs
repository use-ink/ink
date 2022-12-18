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
        ) -> Option<AccountId> {
            use ink::env::call::build_create;

            let result = build_create::<
                DefaultEnvironment,
                constructors_return_value::ConstructorsReturnValueRef,
            >()
            .code_hash(code_hash)
            .gas_limit(0)
            .endowment(0)
            .exec_input(ExecutionInput::new(Selector::new(selector)).push_arg(init_value))
            .salt_bytes(&[0xDE, 0xAD, 0xBE, 0xEF])
            .params()
            .instantiate();

            // NOTE: Right now we can't handle any `LangError` from `instantiate`, we can only tell
            // that our contract reverted (i.e we see error from the Contracts pallet).
            result.ok().map(|id| ink::ToAccountId::to_account_id(&id))
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

            let flipper_constructor = FlipperRef::new_default();
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
                .return_value()
                .expect("Input is valid, call must not fail.");

            let invalid_selector = [0x00, 0x00, 0x00, 0x00];
            let call = build_message::<CallBuilderTestRef>(contract_acc_id)
                .call(|contract| contract.call(flipper_acc_id, invalid_selector));
            let call_result = client
                .call(&ink_e2e::charlie(), call, 0, None)
                .await
                .expect("Calling `call_builder::call` failed");

            let flipper_result = call_result
                .return_value()
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
                .return_value()
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
                .expect("Client failed to call `call_builder::call_instantiate`.")
                .return_value()
                .expect("Dispatching `call_builder::call_instantiate` failed.");

            assert!(
                call_result.is_some(),
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
                .return_value()
                .expect("Dispatching `call_builder::call_instantiate` failed.");

            assert!(
                call_result.is_none(),
                "Call using invalid selector succeeded, when it should've failed."
            );

            Ok(())
        }
    }
}
