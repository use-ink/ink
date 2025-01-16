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

        #[cfg(any(test, feature = "ink-debug"))]
        #[ink(constructor)]
        pub fn constructor2() -> Self {
            Self {}
        }

        #[cfg(feature = "ink-debug")]
        #[ink(constructor)]
        pub fn constructor3() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message1(&self) {}

        #[ink(message)]
        #[cfg(any(test, feature = "ink-debug"))]
        pub fn message2(&self) {}

        #[ink(message)]
        #[cfg(feature = "ink-debug")]
        pub fn message3(&self) {}
    }
}

fn main() {}
