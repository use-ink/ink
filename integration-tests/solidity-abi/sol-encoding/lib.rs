//! A smart contract to test using Solidity ABI encoding.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod sol_encoding {
    #[ink(storage)]
    #[derive(Default)]
    pub struct SolEncoding {
        value: bool,
    }

    impl SolEncoding {
        #[ink(constructor)]
        pub fn new(value: bool) -> Self {
            Self { value }
        }

        /// Set the value
        #[ink(message)]
        pub fn set_value(&mut self, value: bool) {
            self.value = value;
        }

        /// Set the value
        #[ink(message)]
        pub fn get_value(&self) -> bool {
            self.value
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;
