#![cfg_attr(not(feature = "std"), no_std)]

pub use self::subber::Subber;
use ink_lang as ink;

#[ink::contract]
mod subber {
    use accumulator::Accumulator;

    /// Decreases the underlying `accumulator` value.
    #[ink(storage)]
    pub struct Subber {
        /// The `accumulator` to store the value.
        accumulator: accumulator::Accumulator,
    }

    impl Subber {
        /// Creates a new `subber` from the given `accumulator`.
        #[ink(constructor)]
        pub fn new(accumulator: Accumulator) -> Self {
            Self { accumulator }
        }

        /// Decreases the `accumulator` value by some amount.
        #[ink(message)]
        pub fn dec(&mut self, by: i32) {
            self.accumulator.inc(-by)
        }
    }
}
