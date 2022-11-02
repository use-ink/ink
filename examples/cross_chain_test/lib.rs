#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod cross_chain_test {

    #[ink(storage)]
    pub struct CrossChainTest {}

    impl CrossChainTest {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn call(&mut self, address: AccountId, selector: [u8; 4]) {
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
                .fire();

            ink::env::debug_println!("cross_contract::call output: {:?}", &result);
        }
    }
}
