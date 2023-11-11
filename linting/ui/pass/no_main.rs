#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod no_main {
    #[ink(storage)]
    pub struct NoMain {}
    impl NoMain {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }
        #[ink(message)]
        pub fn do_nothing(&mut self) {}
    }
}

fn main() {}
