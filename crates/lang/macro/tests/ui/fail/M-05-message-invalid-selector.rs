use ink_lang as ink;

#[ink::contract]
mod message_invalid_selector {
    #[ink(storage)]
    pub struct MessageInvalidSelector {}

    impl MessageInvalidSelector {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message, selector = "0x00")]
        pub fn invalid_selector(&self) {}
    }
}

fn main() {}
