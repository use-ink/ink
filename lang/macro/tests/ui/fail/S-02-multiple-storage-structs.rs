use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod multiple_storage_structs {
    #[ink(storage)]
    struct FirstStorageStruct {}

    // ink! currently doesn't allow for multiple #[ink(storage)] structs
    #[ink(storage)]
    struct SecondStorageStruct {}

    impl FirstStorageStruct {
        #[ink(constructor)]
        fn constructor1() -> Self {
            Self {}
        }

        #[ink(message)]
        fn message1(&self) {}
    }

    impl SecondStorageStruct {
        #[ink(constructor)]
        fn constructor2() -> Self {
            Self {}
        }

        #[ink(message)]
        fn message2(&self) {}
    }
}

fn main() {}
