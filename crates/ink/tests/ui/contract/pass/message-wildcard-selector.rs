#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message, selector = @)]
        pub fn message_1(&self) {}

        #[ink(message, selector = _)]
        pub fn message_2(&self) {}
    }
}

fn main() {}
