#![allow(unexpected_cfgs)]

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[ink(event, name = "MyEvent")]
    #[ink(
        signature_topic = "1111111111111111111111111111111111111111111111111111111111111111"
    )]
    pub struct Event {
        #[ink(topic)]
        pub topic: [u8; 32],
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

fn main() {
    // Custom signature topic (i.e `signature_topic = "..."`) takes precedence over `name`
    // override
    const SIGNATURE_TOPIC: [u8; 32] = [0x11u8; 32];
    assert_eq!(
        <contract::Event as ink::env::Event>::SIGNATURE_TOPIC,
        Some(SIGNATURE_TOPIC)
    );
}
