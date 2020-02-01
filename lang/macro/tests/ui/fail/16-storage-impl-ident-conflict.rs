use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod noop {
    // This test ensures that ink! impl blocks are always
    // implemented on the only storage struct definition.

    #[ink(storage)]
    struct StorageStruct {}

    // This ink! impl block is okay.
    impl StorageStruct {
        #[ink(constructor)]
        fn new1(&mut self) {}

        #[ink(message)]
        fn do_something1(&self) {}
    }

    // Missing the #[ink(storage)] attribute on purpose.
    struct NonStorageStruct {}

    // This ink! impl block is invalid in that it implements
    // the messages and constructors for a non-existing ink!
    // storage struct. We expect a failure here.
    impl NonStorageStruct {
        #[ink(constructor)]
        fn new2(&mut self) {}

        #[ink(message)]
        fn do_something2(&self) {}
    }
}

fn main() {}
