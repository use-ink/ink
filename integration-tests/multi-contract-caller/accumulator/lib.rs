#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::accumulator::{
    Accumulator,
    AccumulatorRef,
};

#[ink::contract]
pub mod accumulator {
    /// Holds a simple `i32` value that can be incremented and decremented.
    #[ink(storage)]
    pub struct Accumulator {
        value: i32,
    }

    impl Accumulator {
        /// Initializes the value to the initial value.
        #[ink(constructor, payable)]
        pub fn new(init_value: i32) -> Self {
            Self { value: init_value }
        }

        /// Mutates the internal value.
        #[ink(message)]
        pub fn inc(&mut self, by: i32) {
            self.value = self.value.checked_add(by).unwrap();
        }

        /// Returns the current state.
        #[ink(message)]
        pub fn get(&self) -> i32 {
            self.value
        }
    }
}
