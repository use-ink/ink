#![cfg_attr(not(feature = "std"), no_std)]

pub use self::adder::{
    Adder,
    AdderRef,
};

use ink_lang as ink;

#[ink::contract]
mod adder {
    use accumulator::AccumulatorRef;

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

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
        use super::*;
        use accumulator::{Accumulator, AccumulatorRef};

        // register Accumulator & Adder
        let hash1 = ink_env::Hash::try_from([10u8; 32]).unwrap();
        let hash2 = ink_env::Hash::try_from([20u8; 32]).unwrap();
        ink_env::test::register_contract::<ink_env::DefaultEnvironment, Accumulator>(
            hash1.clone()
        );
        ink_env::test::register_contract::<ink_env::DefaultEnvironment, Adder>(
            hash2.clone()
        );

        let acc = AccumulatorRef::new(0)
            .code_hash(hash1.clone())
            .endowment(0)
            .salt_bytes([0u8; 0])
            .instantiate()
            .expect("failed at instantiating the `AccumulatorRef` contract");
        let mut adder = AdderRef::new(acc.clone())
            .code_hash(hash2.clone())
            .endowment(0)
            .salt_bytes([0u8; 0])
            .instantiate()
            .expect("failed at instantiating the `AdderRef` contract");

        assert_eq!(acc.get(), 0);
        adder.inc(1);
        assert_eq!(acc.get(), 1);
    }
}
