//! A smart contract to test using RLP encoding for EVM compatibility.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract(abi_encoding = "rlp")]
mod rlp {
    use ink::prelude::{
        format,
        string::String,
    };

    #[ink(storage)]
    #[derive(Default)]
    pub struct Rlp {
        value: bool,
    }

    impl Rlp {
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
