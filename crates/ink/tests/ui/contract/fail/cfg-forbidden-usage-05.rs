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
        #[cfg(target_pointer_width = "32")]
        pub fn message2(&self) {}
    }
}

fn main() {}
