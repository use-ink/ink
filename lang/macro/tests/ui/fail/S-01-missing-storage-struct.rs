use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod missing_storage_struct {
    // We are missing the #[ink(storage)] attribute here
    struct MissingStorageStruct {}

    impl MissingStorageStruct {
        #[ink(constructor)]
        fn constructor() -> Self {}

        #[ink(message)]
        fn message(&self) {}
    }
}

fn main() {}
