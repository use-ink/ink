#![cfg_attr(not(feature = "std"), no_main)]

#[ink::contract]
pub mod primitive_topic {

    #[ink(event)]
    pub struct Transaction {
        #[ink(topic)]
        src: Option<AccountId>,
        #[ink(topic)]
        dst: Option<AccountId>,
        // Good: no topic annotation
        value_1: Balance,
        // Good: suppressed warning
        #[ink(topic)]
        #[cfg_attr(dylint_lib = "ink_linting", allow(primitive_topic))]
        value_2: Balance,
    }

    #[ink(storage)]
    pub struct PrimitiveTopic {}

    impl PrimitiveTopic {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }
        #[ink(message)]
        pub fn do_nothing(&mut self) {}
    }
}

fn main() {}
