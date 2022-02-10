#![cfg_attr(not(feature = "std"), no_std)]

pub use self::accumulator::{
    Accumulator,
    AccumulatorRef,
    AccumulatorTrait,
};

use ink_lang as ink;

#[ink::contract]
pub mod accumulator {
    /// Allows to mutate and get the current value.
    ///
    /// # Note
    ///
    /// That functionality is moved to the trait to show how it can be
    /// done and how it can be used for cross-contract calls. It is only an example.
    #[ink_lang::trait_definition]
    pub trait AccumulatorTrait {
        /// Mutates the internal value.
        #[ink(message)]
        fn inc(&mut self, by: i32);

        /// Returns the current state.
        #[ink(message)]
        fn value(&self) -> i32;
    }

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
    }

    impl AccumulatorTrait for Accumulator {
        /// Mutates the internal value.
        #[ink(message)]
        fn inc(&mut self, by: i32) {
            self.value += by;
        }

        /// Returns the current state.
        #[ink(message)]
        fn value(&self) -> i32 {
            self.value
        }
    }
}
