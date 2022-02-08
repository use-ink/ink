#![cfg_attr(not(feature = "std"), no_std)]

pub use self::adder::{
    Adder,
    AdderRef,
};

use ink_lang as ink;

#[ink::contract]
mod adder {
    use accumulator::{
        AccumulatorRef,
        AccumulatorTrait,
    };

    /// Increments the underlying `accumulator` value.
    #[ink(storage)]
    pub struct Adder {
        /// The `accumulator` to store the value.
        accumulator: AccumulatorRef,
    }

    impl Adder {
        /// Creates a new `adder` from the given `accumulator`.
        #[ink(constructor)]
        pub fn new(accumulator: AccumulatorRef) -> Self {
            Self { accumulator }
        }

        /// Increases the `accumulator` value by some amount.
        #[ink(message)]
        pub fn inc(&mut self, by: i32) {
            self.accumulator.inc(by)
        }
    }
}
