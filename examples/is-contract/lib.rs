//! A smart contract which demonstrates behavior of the `self.env().is_contract()` function.
//! It checks whether a specified account_id is a contract

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default)]

use ink_lang as ink;

#[ink::contract]
pub mod is_contract {
    /// No storage is needed for this simple contract.
    #[ink(storage)]
    pub struct IsContract {}

    impl IsContract {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Checks if the specified account_id belogns to a contract
        ///
        /// # Errors
        /// TBD
        #[ink(message)]
        pub fn is_contract(&mut self, account_id: AccountId) -> bool {
            ink_env::debug_println!("checking account_id: {:?}", account_id);

            self.env().is_contract(account_id)
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
    }
}
