use ink_lang as ink;

#[ink::event_definition]
pub struct Event0 {}

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[ink(event)]
    type Event0 = super::Event0;

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
