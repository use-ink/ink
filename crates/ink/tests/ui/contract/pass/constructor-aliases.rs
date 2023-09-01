#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    pub type MyTypeAlias = Contract;
    pub type MyResultAlias = Result<MyTypeAlias, Error>;

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        Foo,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> MyTypeAlias {
            Self {}
        }

        #[ink(constructor)]
        pub fn constructor_result() -> Result<MyTypeAlias, Error> {
            Ok(Self {})
        }

        #[ink(constructor)]
        pub fn constructor_result_alias() -> MyResultAlias {
            Ok(Self {})
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
