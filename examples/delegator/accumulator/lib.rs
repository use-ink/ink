#![cfg_attr(not(feature = "std"), no_std)]

pub use self::accumulator::{Accumulator, AccumulatorRef};

use ink_lang as ink;

#[ink::contract]
pub mod accumulator {
    /// Holds a simple `i32` value that can be incremented and decremented.
    #[ink(storage)]
    pub struct Accumulator {
        value: i32,
    }

    impl Accumulator {
        /// Initializes the value to the initial value.
        #[ink(constructor)]
        pub fn new(init_value: i32) -> Self {
            Self { value: init_value }
        }

        /// Mutates the internal value.
        #[ink(message)]
        pub fn inc(&mut self, by: i32) {
            self.value += by;
        }

        /// Returns the current state.
        #[ink(message)]
        pub fn get(&self) -> i32 {
            self.value
        }
    }
}
