//! A smart contract which demonstrates behavior of the `self.env().is_contract()`
//! and `self.env().caller_is_origin()` functions.
//! It checks whether a specified account_id is a contract

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default)]

use ink_lang as ink;

#[ink::contract]
pub mod is_contract {
    use ink_env::call::{
        build_call,
        utils::{
            EmptyArgumentList,
            ReturnType,
        },
        ExecutionInput,
        Selector,
    };

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    /// Error types
    pub enum Error {
        SomeError,
    }

    /// Event emitted when Winning block is detected.
    #[ink(event)]
    pub struct AccountIsContract {
        account_id: AccountId,
        is_contract: bool,
    }

    /// Event emitted when a winner is detected.
    #[ink(event)]
    pub struct CallerIsOrigin {
        is_origin: bool,
    }

    /// No storage is needed for this simple contract.
    #[ink(storage)]
    pub struct IsContract {}

    impl IsContract {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Cross contract invocation method  
        fn invoke_contract(
            &self,
            contract: AccountId,
            input: ExecutionInput<EmptyArgumentList>,
        ) {
            let params = build_call::<Environment>()
                .callee(contract)
                .exec_input(input)
                .returns::<ReturnType<Result<(), Error>>>();

            match params.fire() {
                Ok(_v) => {}
                Err(e) => {
                    match e {
                        ink_env::Error::CodeNotFound | ink_env::Error::NotCallable => {
                            // Our recipient wasn't a smart contract, so there's nothing more for
                            // us to do
                            let msg = ink_prelude::format!(
                                "Recipient at {:#04X?} from is not a smart contract ({:?})",
                                contract,
                                e
                            );
                            panic!("{}", msg)
                        }
                        _ => {
                            // We got some sort of error from the call to our recipient smart
                            // contract, and as such we must revert this call
                            let msg = ink_prelude::format!(
                                "Got error \"{:?}\" while trying to call {:?}",
                                e,
                                contract,
                            );
                            panic!("{}", msg)
                        }
                    }
                }
            }
        }

        /// Checks if the specified account_id belogns to a contract
        #[ink(message, selector = 0xBABEFEED)]
        pub fn is_contract(&mut self, account_id: AccountId) -> bool {
            ink_env::debug_println!(
                "checking if account_id is contract: {:?}",
                account_id
            );

            let is_contract = self.env().is_contract(account_id);

            self.env().emit_event(AccountIsContract {
                account_id: account_id.clone(),
                is_contract: is_contract.clone(),
            });

            is_contract
        }

        /// Checks if the caller is the origin of the whole call stack.
        #[ink(message)]
        pub fn caller_is_origin(&mut self) -> bool {
            ink_env::debug_println!("checking if caller is the origin",);

            let is_origin = self.env().caller_is_origin();

            self.env().emit_event(CallerIsOrigin {
                is_origin: is_origin.clone(),
            });

            is_origin
        }

        /// Checks if the caller is the origin of the whole call stack.
        #[ink(message)]
        pub fn call_contract(&mut self, address: AccountId) {
            let selector = Selector::new([0xBA, 0xBE, 0xFE, 0xED]);
            let input = ExecutionInput::new(selector);
            self.invoke_contract(address, input);
        }
    }

    #[cfg(not(feature = "ink-experimental-engine"))]
    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        fn accounts() -> ink_env::test::DefaultAccounts<Environment> {
            ink_env::test::default_accounts::<Environment>().unwrap()
        }

        #[ink::test]
        fn is_contract_works() {
            // to be honest, this is just a boilerplate
            // since off_chain env does not support contracts
            assert!(!ink_env::is_contract::<Environment>(accounts().alice));
        }

        #[ink::test]
        fn caller_is_origin_works() {
            // to be honest, this is just a boilerplate
            // since off_chain env does not support contracts
            assert!(ink_env::caller_is_origin::<Environment>());
        }
    }
}
