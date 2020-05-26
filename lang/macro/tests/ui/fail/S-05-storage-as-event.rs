use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod storage_as_event {
    #[ink(storage)]
    #[ink(event)] // We cannot have #[ink(event)] if we already have #[ink(storage)]
    struct StorageAsEvent {}

    impl StorageAsEvent {
        #[ink(constructor)]
        fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        fn message(&self) {}
    }
}

fn main() {}
