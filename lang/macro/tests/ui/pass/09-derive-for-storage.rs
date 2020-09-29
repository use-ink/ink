use ink_lang as ink;

#[ink::contract]
mod derive_for_storage {
    #[ink(storage)]
    #[derive(Default)]
    pub struct DeriveForStorage {}

    impl DeriveForStorage {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Default::default()
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
