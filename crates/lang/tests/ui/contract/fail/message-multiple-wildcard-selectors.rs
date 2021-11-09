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

        #[ink(message, selector = "_")]
        pub fn message1(&self) {}

        #[ink(message, selector = "_")]
        pub fn message2(&self) {}
    }
}

fn main() {}
