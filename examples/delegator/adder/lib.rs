#![cfg_attr(not(feature = "std"), no_std)]

pub use self::adder::{
    Adder,
    AdderRef,
};

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
    use ink::primitives::Hash;

    #[test]
    fn cross_contract_call_works_off_chain() {
        use super::*;
        use accumulator::{
            Accumulator,
            AccumulatorRef,
        };

        // register Accumulator & Adder
        let hash1 = Hash::from([10u8; 32]);
        let hash2 = Hash::from([20u8; 32]);
        ink::env::test::register_contract::<Accumulator>(hash1.as_ref());
        ink::env::test::register_contract::<Adder>(hash2.as_ref());

        let acc = AccumulatorRef::new(0)
            .code_hash(hash1)
            .endowment(0)
            .salt_bytes([0u8; 0])
            .instantiate();
        let mut adder = AdderRef::new(acc.clone())
            .code_hash(hash2)
            .endowment(0)
            .salt_bytes([0u8; 0])
            .instantiate();

        assert_eq!(acc.get(), 0);
        adder.inc(1);
        assert_eq!(acc.get(), 1);
    }
}
