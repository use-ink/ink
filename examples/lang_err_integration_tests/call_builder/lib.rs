//! # Integration Tests for `LangError`
//!
//! This contract is used to ensure that the behavior around `LangError`s works as expected.
//!
//! It makes use of ink!'s end-to-end testing features, so ensure that you have a node which
//! includes the Contract's pallet running alongside your tests.

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod call_builder {

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
            use ink::env::{
                call::{
                    build_call,
                    Call,
                    ExecutionInput,
                    Selector,
                },
                DefaultEnvironment,
            };

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
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(additional_contracts = "../flipper/Cargo.toml")]
        async fn e2e_invalid_selector_can_be_handled(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            use call_builder::contract_types::ink_primitives::{
                types::AccountId as E2EAccountId,
                LangError as E2ELangError,
            };

            let constructor = call_builder::constructors::new();
            let contract_acc_id = client
                .instantiate(&mut ink_e2e::charlie(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let flipper_constructor = flipper::constructors::default();
            let flipper_acc_id = client
                .instantiate(&mut ink_e2e::charlie(), flipper_constructor, 0, None)
                .await
                .expect("instantiate `flipper` failed")
                .account_id;

            let get_call_result = client
                .call(
                    &mut ink_e2e::charlie(),
                    flipper_acc_id.clone(),
                    flipper::messages::get(),
                    0,
                    None,
                )
                .await
                .expect("Calling `flipper::get` failed");
            let initial_value = get_call_result
                .value
                .expect("Input is valid, call must not fail.");

            let flipper_ink_acc_id = E2EAccountId(flipper_acc_id.clone().into());
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
                    flipper::messages::get(),
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
    }
}
