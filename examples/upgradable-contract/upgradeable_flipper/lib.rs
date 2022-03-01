#![cfg_attr(not(feature = "std"), no_std)]

mod upgradeable;

use ink_lang as ink;

#[ink::contract]
pub mod flipper {
    use crate::upgradeable::{
        NotInitialized,
        Upgradable,
    };

    #[ink(storage)]
    pub struct Flipper {
        /// The field is `Upgradable`, which means if the field is not initialized, it will be.
        ///
        /// By default ink! throw an error that field is not initialized.
        /// With that wrapper, you can initialize the field later during the method execution,
        /// not in the constructor.
        value: Upgradable<bool, NotInitialized>,
    }

    impl Flipper {
        /// Creates a new flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self {
                value: Upgradable::new(init_value),
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
