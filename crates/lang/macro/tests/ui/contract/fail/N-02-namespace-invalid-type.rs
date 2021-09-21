use ink_lang as ink;

#[ink::contract]
mod invalid_namespace_identifier {
    #[ink(storage)]
    pub struct MyStorage {}

    #[ink(namespace = true)]
    impl MyStorage {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
