use ink_lang as ink;

#[ink::contract]
mod event_as_storage {
    #[ink(event)]
    #[ink(storage)] // We cannot have #[ink(storage)] if we already have #[ink(event)]
    pub struct EventAsStorage {}

    impl EventAsStorage {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
