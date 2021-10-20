use ink_lang as ink;

#[ink::contract]
mod contract {
    #[ink(storage)]
    #[ink(unknown_or_unsupported)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
