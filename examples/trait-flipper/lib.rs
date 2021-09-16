#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default)]

use ink_lang as ink;

#[ink::trait_definition]
pub trait Flip {
    /// Flips the current value of the Flipper's boolean.
    #[ink(message)]
    fn flip(&mut self);

    /// Returns the current value of the Flipper's boolean.
    #[ink(message)]
    fn get(&self) -> bool;
}

#[ink::contract]
pub mod flipper {
    use super::Flip;

    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { value: Default::default() }
        }
    }

    impl Flip for Flipper {
        #[ink(message)]
        fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        fn get(&self) -> bool {
            self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn default_works() {
            let flipper = Flipper::new();
            assert!(!flipper.get());
        }

        #[ink::test]
        fn it_works() {
            let mut flipper = Flipper::new();
            // Can call using universal call syntax using the trait.
            assert!(!<Flipper as Flip>::get(&flipper));
            <Flipper as Flip>::flip(&mut flipper);
            // Normal call syntax possible to as long as the trait is in scope.
            assert!(flipper.get());
        }
    }
}
