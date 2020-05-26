use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod missing_message {
    #[ink(storage)]
    struct MissingMessage {}

    impl MissingMessage {
        #[ink(constructor)]
        fn constructor() -> Self {
            Self {}
        }
    }
}

fn main() {}
