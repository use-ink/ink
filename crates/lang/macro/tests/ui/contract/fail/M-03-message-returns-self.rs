use ink_lang as ink;

#[ink::contract]
mod message_returns_self {
    #[ink(storage)]
    pub struct MessageReturnsSelf {}

    impl MessageReturnsSelf {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn returns_self(&self) -> Self {}
    }
}

fn main() {}
