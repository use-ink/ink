#![allow(unexpected_cfgs)]

#[ink::contract]
mod contract {
    #[ink(storage)]
    #[ink(event)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
