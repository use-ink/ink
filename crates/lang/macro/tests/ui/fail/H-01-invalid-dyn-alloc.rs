use ink_lang as ink;

#[ink::contract(dynamic_storage_allocator = "foo")]
mod invalid_version {
    #[ink(storage)]
    pub struct InvalidVersion {}

    impl InvalidVersion {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
