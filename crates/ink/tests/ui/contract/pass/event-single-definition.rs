#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[ink::event_definition]
    pub enum Event {
        Event0 {}
    }

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
