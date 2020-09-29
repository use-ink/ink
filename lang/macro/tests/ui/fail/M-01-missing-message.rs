use ink_lang as ink;

#[ink::contract]
mod missing_message {
    #[ink(storage)]
    pub struct MissingMessage {}

    impl MissingMessage {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }
    }
}

fn main() {}
