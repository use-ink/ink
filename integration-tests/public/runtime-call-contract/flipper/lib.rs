#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use flipper::{
    Flipper,
    FlipperRef,
};

#[ink::contract]
mod flipper {
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

        /// Returns the current balance of the Flipper.
        #[ink(message)]
        pub fn get_contract_balance(&self) -> Balance {
            self.env().balance()
        }
    }
}
