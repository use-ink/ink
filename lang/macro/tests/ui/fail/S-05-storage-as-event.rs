use ink_lang as ink;

#[ink::contract]
mod storage_as_event {
    #[ink(storage)]
    #[ink(event)] // We cannot have #[ink(event)] if we already have #[ink(storage)]
    pub struct StorageAsEvent {}

    impl StorageAsEvent {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
