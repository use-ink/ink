use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod missing_message_self_arg {
    #[ink(storage)]
    struct MissingMessageSelfArg {}

    impl MissingMessage {
        #[ink(constructor)]
        fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        fn missing_self_arg() {}
    }
}

fn main() {}
