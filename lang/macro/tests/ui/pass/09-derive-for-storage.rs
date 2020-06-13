use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod derive_for_storage {
    #[ink(storage)]
    #[derive(Default)]
    struct DeriveForStorage {}

    impl DeriveForStorage {
        #[ink(constructor)]
        fn constructor() -> Self {
            Default::default()
        }

        #[ink(message)]
        fn message(&self) {}
    }
}

fn main() {}
