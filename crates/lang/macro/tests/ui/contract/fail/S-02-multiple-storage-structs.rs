use ink_lang as ink;

#[ink::contract]
mod multiple_storage_structs {
    #[ink(storage)]
    pub struct FirstStorageStruct {}

    // ink! currently does not allow for multiple #[ink(storage)] structs
    #[ink(storage)]
    pub struct SecondStorageStruct {}

    impl FirstStorageStruct {
        #[ink(constructor)]
        pub fn constructor1() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message1(&self) {}
    }

    impl SecondStorageStruct {
        #[ink(constructor)]
        pub fn constructor2() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message2(&self) {}
    }
}

fn main() {}
