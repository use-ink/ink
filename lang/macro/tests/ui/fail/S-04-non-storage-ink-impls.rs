use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod non_storage_ink_impls {
    // This test ensures that ink! impl blocks are always
    // implemented on the only storage struct definition.

    #[ink(storage)]
    struct StorageStruct {}

    // This ink! impl block is okay.
    impl StorageStruct {
        #[ink(constructor)]
        fn constructo1() -> Self {}

        #[ink(message)]
        fn message1(&self) {}
    }

    // Missing the #[ink(storage)] attribute on purpose.
    struct NonStorageStruct {}

    // This ink! impl block is invalid in that it implements
    // the messages and constructors for a non-existing ink!
    // storage struct. We expect a failure here.
    impl NonStorageStruct {
        #[ink(constructor)]
        fn constructor2() -> Self {}

        #[ink(message)]
        fn message2(&self) {}
    }
}

fn main() {}
