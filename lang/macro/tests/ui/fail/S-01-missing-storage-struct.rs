use ink_lang as ink;

#[ink::contract]
mod missing_storage_struct {
    // We are missing the #[ink(storage)] attribute here
    pub struct MissingStorageStruct {}

    impl MissingStorageStruct {
        #[ink(constructor)]
        pub fn constructor() -> Self {}

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
