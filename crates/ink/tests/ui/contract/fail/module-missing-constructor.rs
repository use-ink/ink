#![allow(unexpected_cfgs)]

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
