#![allow(unexpected_cfgs)]

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[cfg(any(test, target_family = "wasm", feature = "std"))]
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message1(&self) {}

        #[ink(message)]
        pub fn message2(&self) {}
    }
}

fn main() {}
