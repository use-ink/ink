use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod missing_message_returns_self {
    #[ink(storage)]
    struct MissingMessageReturnsSelf {}

    impl MissingMessageReturnsSelf {
        #[ink(constructor)]
        fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        fn missing_self_arg(&self) -> Self {}
    }
}

fn main() {}
