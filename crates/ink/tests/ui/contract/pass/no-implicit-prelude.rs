#![no_implicit_prelude]

#[::ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(constructor)]
        pub fn constructor_result() -> ::core::result::Result<Self, ()> {
            ::core::result::Result::Ok(Self {})
        }

        #[ink(message)]
        pub fn message(&self) {}

        #[ink(message)]
        pub fn message_result(&self) -> ::core::result::Result<(), ()> {
            ::core::result::Result::Ok(())
        }
    }
}

fn main() {}
