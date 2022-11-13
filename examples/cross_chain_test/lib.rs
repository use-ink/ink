#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod cross_chain_test {

    #[ink(storage)]
    #[derive(Default)]
    pub struct CrossChainTest {}

    impl CrossChainTest {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn call(
            &mut self,
            address: AccountId,
            selector: [u8; 4],
        ) -> Result<(), ::ink::LangError> {
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
                // .returns::<()>()
                .fire()
                .expect("seal error");

            ink::env::debug_println!("cross_contract::call output: {:?}", &result);
            match result {
                Ok(_) => Ok(()),
                Err(e @ ink::LangError::CouldNotReadInput) => {
                    ink::env::debug_println!("CouldNotReadInput");
                    Err(e)
                }
                Err(_) => unimplemented!(),
            }
        }
    }

    #[cfg(test)]
    mod e2e_tests {
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(additional_contracts = "../flipper/Cargo.toml")]
        async fn e2e_cross_chain_test(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            use cross_chain_test::contract_types::ink_primitives::{
                types::AccountId as E2EAccountId,
                LangError as E2ELangError,
            };

            let constructor = cross_chain_test::constructors::new();
            let contract_acc_id = client
                .instantiate(&mut ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let flipper_constructor = flipper::constructors::default();
            let flipper_acc_id = client
                .instantiate(&mut ink_e2e::alice(), flipper_constructor, 0, None)
                .await
                .expect("instantiate `flipper` failed")
                .account_id;

            // --- Can Flip Correctly ---

            let get_call_result = client
                .call(
                    &mut ink_e2e::bob(),
                    flipper_acc_id.clone(),
                    flipper::messages::get(),
                    0,
                    None,
                )
                .await
                .expect("Calling `flipper::get` failed");
            let initial_value = get_call_result
                .value
                .expect("Shouldn't fail since input is valid");

            let flip_call_result = client
                .call(
                    &mut ink_e2e::bob(),
                    flipper_acc_id.clone(),
                    flipper::messages::flip(),
                    0,
                    None,
                )
                .await
                .expect("Calling `flipper::flip` failed");
            assert!(flip_call_result.value.is_ok());

            let get_call_result = client
                .call(
                    &mut ink_e2e::bob(),
                    flipper_acc_id.clone(),
                    flipper::messages::get(),
                    0,
                    None,
                )
                .await
                .expect("Calling `flipper::get` failed");
            let flipped_value = get_call_result
                .value
                .expect("Shouldn't fail since input is valid");
            assert!(flipped_value == !initial_value);

            // --- Invalid Selector Can Be Handled ---

            let get_call_result = client
                .call(
                    &mut ink_e2e::bob(),
                    flipper_acc_id.clone(),
                    flipper::messages::get(),
                    0,
                    None,
                )
                .await
                .expect("Calling `flipper::get` failed");
            let initial_value = get_call_result
                .value
                .expect("Shouldn't fail since input is valid");

            let flipper_ink_acc_id = E2EAccountId(flipper_acc_id.clone().into());
            let invalid_selector = [0x00, 0x00, 0x00, 0x00];
            let call_result = client
                .call(
                    &mut ink_e2e::bob(),
                    contract_acc_id.clone(),
                    cross_chain_test::messages::call(
                        flipper_ink_acc_id,
                        invalid_selector,
                    ),
                    0,
                    None,
                )
                .await
                .expect("Calling `cross_chain_test::call` failed");

            let flipper_result = call_result
                .value
                .expect("Call to `cross_chain_test::call` failed");

            // TODO: Need to figure out how to derive `PartialEq` for `e2e::LangError`
            match flipper_result {
                Ok(_) => panic!("should've been an error"),
                Err(E2ELangError::CouldNotReadInput) => {}
                // TODO: Need to figure out how to make `e2e::LangError` `non_exhaustive`
                #[allow(unreachable_patterns)]
                Err(_) => panic!("should've been a different error"),
            };

            let get_call_result = client
                .call(
                    &mut ink_e2e::bob(),
                    flipper_acc_id.clone(),
                    flipper::messages::get(),
                    0,
                    None,
                )
                .await
                .expect("Calling `flipper::get` failed");
            let flipped_value = get_call_result
                .value
                .expect("Shouldn't fail since input is valid");
            assert!(flipped_value == initial_value);

            // --- State is Reverted on Message Error ---

            let get_call_result = client
                .call(
                    &mut ink_e2e::bob(),
                    flipper_acc_id.clone(),
                    flipper::messages::get(),
                    0,
                    None,
                )
                .await
                .expect("Calling `flipper::get` failed");
            let initial_value = get_call_result
                .value
                .expect("Shouldn't fail since input is valid");

            let err_flip_call_result = client
                .call(
                    &mut ink_e2e::bob(),
                    flipper_acc_id.clone(),
                    flipper::messages::err_flip(),
                    0,
                    None,
                )
                .await
                .expect("Calling `flipper::err_flip` failed");
            let flipper_result = err_flip_call_result
                .value
                .expect("Call to `flipper::err_flip` failed");
            assert!(flipper_result.is_err());

            let get_call_result = client
                .call(
                    &mut ink_e2e::bob(),
                    flipper_acc_id.clone(),
                    flipper::messages::get(),
                    0,
                    None,
                )
                .await
                .expect("Calling `flipper::get` failed");
            let flipped_value = get_call_result
                .value
                .expect("Shouldn't fail since input is valid");
            assert!(flipped_value == initial_value);

            Ok(())
        }
    }
}
