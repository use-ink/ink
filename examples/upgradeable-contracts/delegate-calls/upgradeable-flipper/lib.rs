//! This is an example of an upgradable `Flipper`, that can be deployed by the developer
//! and used with `Proxy` from the `upgradeable_contract` crate.
//! The calls from the `Proxy` contract can be delegated to that contract.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod flipper {
    use ink_storage::traits::OnCallInitializer;

    /// The `Flipper` doesn't use the manual storage key.
    /// That means that it is stored under the default zero storage key.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Flipper {
        value: bool,
    }

    /// By default ink! would throw an error that the field is not initialized.
    /// But if the contract implements `ink_storage::traits::OnCallInitializer`, then it will
    /// be initialized later in the `OnCallInitializer::initialize` during the method execution,
    /// not in the constructor.
    impl OnCallInitializer for Flipper {
        fn initialize(&mut self) {
            // Let's initialize it with `false` by default
            self.value = false;
        }
    }

    impl Flipper {
        /// Creates a new flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Flips the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Returns the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn default_works() {
            let flipper = Flipper::default();
            assert!(!flipper.get());
        }

        #[ink::test]
        fn it_works() {
            let mut flipper = Flipper::new(false);
            assert!(!flipper.get());
            flipper.flip();
            assert!(flipper.get());
        }
    }
}
