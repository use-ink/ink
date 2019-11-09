#![feature(proc_macro_hygiene)]

use ink_lang2 as ink;

#[ink::contract(version = "foo")]
mod invalid_version {
    #[ink(storage)]
    struct InvalidVersion {}

    impl Noop {
        #[ink(constructor)]
        fn new(&mut self) {}

        #[ink(message)]
        fn noop(&self) {}
    }
}

fn main() {}
