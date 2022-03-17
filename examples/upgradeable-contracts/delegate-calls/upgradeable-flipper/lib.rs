//! This is an example of an upgradable `Flipper`, that can be deployed by the developer
//! and used with `Proxy` from the `upgradeable_contract` crate.
//! The calls from the `Proxy` contract can be delegated to that contract.

#![cfg_attr(not(feature = "std"), no_std)]

mod upgradeable;

use ink_lang as ink;

#[ink::contract]
pub mod flipper {
    use crate::upgradeable::{
        NotInitialized,
        Upgradeable,
    };

    #[ink(storage)]
    pub struct Flipper {
        /// The field is `Upgradeable`, which means if the field is not initialized, it will be.
        ///
        /// By default ink! would throw an error that the field is not initialized.
        /// With that wrapper, you can initialize the field later during the method execution,
        /// not in the constructor.
        value: Upgradeable<bool, NotInitialized>,
    }

    impl Flipper {
        /// Creates a new flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self {
                value: Upgradeable::new(init_value),
            }
        }

        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// Flips the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn flip(&mut self) {
            *self.value = !*self.value;
        }

        /// Returns the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn get(&self) -> bool {
            *self.value
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
