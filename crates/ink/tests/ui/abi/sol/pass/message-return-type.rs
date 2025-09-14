#![allow(unexpected_cfgs)]

#[ink::error]
pub struct Error;

#[ink::contract]
mod contract {
    use super::Error;

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message_no_return(&self) {}

        #[ink(message)]
        pub fn message_return_non_result(&self) -> bool {
            true
        }

        #[ink(message)]
        pub fn message_return_result(&self) -> Result<u8, Error> {
            Ok(0)
        }
    }
}

fn main() {}
