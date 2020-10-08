use ink_lang as ink;

#[ink::contract]
mod missing_message_self_arg {
    #[ink(storage)]
    pub struct MissingMessageSelfArg {}

    impl MissingMessage {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn missing_self_arg() {}
    }
}

fn main() {}
