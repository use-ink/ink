#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::subber::{
    Subber,
    SubberRef,
};

#[ink::contract]
mod subber {
    use accumulator::AccumulatorRef;

    /// Decreases the underlying `accumulator` value.
    #[ink(storage)]
    pub struct Subber {
        /// The `accumulator` to store the value.
        accumulator: AccumulatorRef,
    }

    impl Subber {
        /// Creates a new `subber` from the given `accumulator`.
        #[ink(constructor, payable)]
        pub fn new(accumulator: AccumulatorRef) -> Self {
            Self { accumulator }
        }

        /// Decreases the `accumulator` value by some amount.
        #[ink(message)]
        pub fn dec(&mut self, by: i32) {
            self.accumulator.inc(0i32.checked_sub(by).unwrap())
        }
    }
}
