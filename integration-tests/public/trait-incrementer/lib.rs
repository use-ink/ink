#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
pub mod incrementer {
    use traits::{
        Increment,
        Reset,
    };

    /// A concrete incrementer smart contract.
    #[ink(storage)]
    pub struct Incrementer {
        value: u64,
    }

    impl Incrementer {
        /// Creates a new incrementer smart contract initialized with zero.
        #[ink(constructor)]
        pub fn new(init_value: u64) -> Self {
            Self { value: init_value }
        }

        /// Increases the value of the incrementer by an amount.
        #[ink(message)]
        pub fn inc_by(&mut self, delta: u64) {
            self.value = self.value.checked_add(delta).unwrap();
        }
    }

    impl Increment for Incrementer {
        #[ink(message)]
        fn inc(&mut self) {
            self.inc_by(1)
        }

        #[ink(message)]
        fn get(&self) -> u64 {
            self.value
        }
    }

    impl Reset for Incrementer {
        #[ink(message)]
        fn reset(&mut self) {
            self.value = 0;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn default_works() {
            let incrementer = Incrementer::new(0);
            assert_eq!(incrementer.get(), 0);
        }

        #[test]
        fn it_works() {
            let mut incrementer = Incrementer::new(0);
            // Can call using universal call syntax using the trait.
            assert_eq!(<Incrementer as Increment>::get(&incrementer), 0);
            <Incrementer as Increment>::inc(&mut incrementer);
            // Normal call syntax possible to as long as the trait is in scope.
            assert_eq!(incrementer.get(), 1);
        }
    }
}
