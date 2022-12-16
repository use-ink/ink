#[ink::contract]
mod contract {
    #[ink(storage)]
    #[derive(Default)]
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

use contract::Contract;

fn main() {
    let contract = Contract::default();
    contract.message();
}
