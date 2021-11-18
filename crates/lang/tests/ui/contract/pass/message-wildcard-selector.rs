use ink_lang as ink;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message_0(&self) {}

        #[ink(message, selector = 1)]
        pub fn message_1(&self) {}

        #[ink(message, selector = _)]
        pub fn message_2(&self) {}
    }
}

fn main() {}
