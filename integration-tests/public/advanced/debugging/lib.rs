//! # Debugging Strategies
//!
//! This contract illustrates a number of strategies for debugging
//! contracts:
//!
//! * Emitting debugging events.
//! * The `pallet-revive` tracing API.
//! * Causing intentional reverts with a return value, in your contract.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod debugging_strategies {
    use flipper::FlipperRef;
    use ink::env::call::{
        ExecutionInput,
        Selector,
        build_call,
        build_create,
    };
    #[cfg(feature = "debug")]
    use ink::prelude::{
        borrow::ToOwned,
        format,
        string::String,
    };

    #[ink::event]
    #[cfg(feature = "debug")]
    pub struct DebugEvent {
        message: String,
    }

    /// Storage of the contract.
    #[ink(storage)]
    #[derive(Default)]
    pub struct DebuggingStrategies {
        value: bool,
    }

    impl DebuggingStrategies {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { value: true }
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            #[cfg(feature = "debug")]
            self.env().emit_event(DebugEvent {
                message: format!("received {:?}", self.env().transferred_value())
                    .to_owned(),
            });
            self.value
        }

        #[ink(message)]
        pub fn intentional_revert(&self) {
            #[cfg(feature = "debug")]
            ink::env::return_value(
                ink::env::ReturnFlags::REVERT,
                &format!("reverting with info: {:?}", self.env().transferred_value()),
            );
        }

        #[ink(message, payable)]
        pub fn instantiate_and_call(&mut self, code_hash: ink::H256) -> bool {
            let create_params = build_create::<FlipperRef>()
                .code_hash(code_hash)
                .endowment(0.into())
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    Abi::Ink,
                    "new"
                ))))
                .returns::<FlipperRef>()
                .params();

            let other: FlipperRef = self.env()
                .instantiate_contract(&create_params)
                // todo
                // we do this to make sure the instantiated contract is always at a
                // different address
                // .salt(self.env().addr().to_vec())
                .unwrap_or_else(|error| {
                    panic!(
                        "Received an error from `pallet-revive` while instantiating: {error:?}"
                    )
                })
                .unwrap_or_else(|error| {
                    panic!("Received a `LangError` while instantiating: {error:?}")
                });

            let call = build_call()
                .call(ink::ToAddr::to_addr(&other))
                .transferred_value(0.into())
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    Abi::Ink,
                    "get"
                ))))
                .returns::<bool>()
                .params();

            self.env()
                .invoke_contract(&call)
                .unwrap_or_else(|env_err| {
                    panic!("Received an error from the Environment: {env_err:?}")
                })
                .unwrap_or_else(|lang_err| panic!("Received a `LangError`: {lang_err:?}"))
        }
    }
}

#[cfg(test)]
mod tests;