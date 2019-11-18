#![feature(proc_macro_hygiene)]

use ink_lang2 as ink;

#[ink::contract(version = "0.1.0")]
mod noop {
    #[ink(storage)]
    struct Noop {}

    impl Noop {
        #[ink(constructor)]
        fn new(&mut self) {
            let _ = &self.__env;
        }

        #[ink(message)]
        fn noop(&self) {
            let _ = &self.__storage;
        }
    }
}

fn main() {}
