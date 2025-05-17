#![allow(unexpected_cfgs)]

#[ink::contract]
pub mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message, selector = 0xe21f37ce)]
        // Solidity selector is `keccak256("message()")` == `0xe21f37ce` == `3793696718`
        pub fn message(&self) {}
    }
}

fn main() {}
