#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(encode, decode, type_info)]
    pub enum Error {
        Foo,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Result<Self, Error> {
            Ok(Self {})
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
