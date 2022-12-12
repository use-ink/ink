#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[ink::event_definition]
    #[ink(storage)]
    pub enum Event {
        Event {},
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
