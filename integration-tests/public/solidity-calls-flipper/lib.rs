#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract(abi = "sol")]
pub mod flipper {
    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        /// Creates a new flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        // solidity compatible selector (`keccack256("flip()")`)
        #[ink(message, selector = 0xcde4efa9)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        pub fn set(&mut self, value: bool) {
            self.value = value;
        }

        #[ink(message)]
        pub fn flip_2(&mut self) {
            self.value = !self.value;
        }

        /// Returns the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }

        // solidity compatible selector (`keccack256("get_2()")`)
        #[ink(message, selector = 0x6d4ce63c)]
        pub fn get_2(&self) -> bool {
            self.value
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;
