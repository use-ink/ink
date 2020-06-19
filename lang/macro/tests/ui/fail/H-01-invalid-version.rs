use ink_lang as ink;

#[ink::contract(version = "foo")]
mod invalid_version {
    #[ink(storage)]
    struct InvalidVersion {}

    impl InvalidVersion {
        #[ink(constructor)]
        fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        fn message(&self) {}
    }
}

fn main() {}
