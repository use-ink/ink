use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod event_as_storage {
    #[ink(event)]
    #[ink(storage)] // We cannot have #[ink(storage)] if we already have #[ink(event)]
    struct EventAsStorage {}

    impl EventAsStorage {
        #[ink(constructor)]
        fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        fn message(&self) {}
    }
}

fn main() {}
