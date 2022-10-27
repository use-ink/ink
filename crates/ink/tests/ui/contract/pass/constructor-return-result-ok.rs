#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
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
