#![allow(unexpected_cfgs)]

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[ink(event, anonymous)]
    pub struct Event {
        #[ink(topic)]
        pub topic_1: [u8; 32],
        #[ink(topic)]
        pub topic_2: [u8; 32],
        #[ink(topic)]
        pub topic_3: [u8; 32],
        #[ink(topic)]
        pub topic_4: [u8; 32],
        #[ink(topic)]
        pub topic_5: [u8; 32],
        pub field_1: u32,
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
