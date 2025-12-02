//! A smart contract which demonstrates behavior of the `self.env().terminate()`
//! function. It terminates itself once `terminate_me()` is called.

#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
pub mod just_terminates {
    /// No storage is needed for this simple contract.
    #[ink(storage)]
    pub struct JustTerminate {}

    impl JustTerminate {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Terminates with the caller as beneficiary.
        #[ink(message)]
        pub fn terminate_me(&mut self) {
            self.env().terminate_contract(self.env().caller());
        }
    }
}

#[cfg(test)]
mod tests;