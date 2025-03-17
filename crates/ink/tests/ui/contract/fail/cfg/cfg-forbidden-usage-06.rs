#![allow(unexpected_cfgs)]

#[ink::contract(keep_attr = "cfg")]
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
        #[cfg(target_os = "wasm")]
        pub fn message2(&self) {}
    }
}

fn main() {}
