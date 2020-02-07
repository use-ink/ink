use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod noop {
    #[ink(storage)]
    struct FirstStorage {}

    // ink! currently doesn't allow for multiple #[ink(storage)] structs
    #[ink(storage)]
    struct SecondStorage {}

    impl FirstStorage {
        #[ink(constructor)]
        fn new(&mut self) {}

        #[ink(message)]
        fn do_something(&self) {}
    }

    impl SecondStorage {
        #[ink(constructor)]
        fn new(&mut self) {}

        #[ink(message)]
        fn do_something(&self) {}
    }
}

fn main() {}
