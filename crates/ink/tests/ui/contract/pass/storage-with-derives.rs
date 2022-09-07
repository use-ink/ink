use contract::Contract;
use ink_lang as ink;

#[ink::contract]
mod contract {
    #[ink(storage)]
    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self::default()
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {
    assert_eq!(Contract::constructor(), Contract::default());
}
