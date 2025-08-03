// `sol_name` is only supported in Solidity ABI compatibility mode.

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message, sol_name = "myMessage")]
        pub fn message(&self) {}
    }
}

fn main() {}
