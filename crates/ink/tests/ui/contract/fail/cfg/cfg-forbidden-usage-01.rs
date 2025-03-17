#![allow(unexpected_cfgs)]

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message1(&self) {}

        #[ink(message)]
        #[cfg(any(test, target_family = "wasm"))]
        pub fn message2(&self) {}
    }
}

fn main() {}
