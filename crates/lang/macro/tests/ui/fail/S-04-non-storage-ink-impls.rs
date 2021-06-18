use ink_lang as ink;

#[ink::contract]
mod non_storage_ink_impls {
    // This test ensures that ink! `impl` blocks are always
    // implemented on the only storage struct definition.

    #[ink(storage)]
    pub struct StorageStruct {}

    // This ink! `impl` block is okay.
    impl StorageStruct {
        #[ink(constructor)]
        pub fn constructor1() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message1(&self) {}
    }

    // Missing the `#[ink(storage)]` attribute on purpose.
    pub struct NonStorageStruct {}

    // This ink! `impl` block is invalid in that it implements
    // the messages and constructors for a non-existing ink!
    // storage struct. We expect a failure here.
    impl NonStorageStruct {
        #[ink(constructor)]
        pub fn constructor2() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message2(&self) {}
    }
}

fn main() {}
