#![no_std]

use pdsl_core::{
    storage,
    memory::format,
};
use pdsl_lang::contract;

contract! {
    /// A simple incrementer contract that can only increment,
    /// compare and return its internal value.
    struct Incrementer {
        /// The current value.
        value: storage::Value<u32>,
    }

    impl Deploy for Incrementer {
        /// Initializes the value to the initial value.
        fn deploy(&mut self, init_value: u32) {
            self.value.set(init_value)
        }
    }

    impl Incrementer {
        /// Flips the current state of our smart contract.
        pub(external) fn inc(&mut self, by: u32) {
            env.println(&format!("Incrementer::inc"));
            self.value += by;
        }

        /// Returns the current state.
        pub(external) fn get(&self) -> u32 {
            env.println("Incrementer::get");
            *self.value
        }

        /// Returns `true` if the internal value is greater than or equal to the provided value.
        pub(external) fn compare(&self, with: u32) -> bool {
            env.println("Incrementer::compare");
            *self.value >= with
        }
    }
}
