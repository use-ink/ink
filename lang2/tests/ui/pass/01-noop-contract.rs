#![feature(proc_macro_hygiene)]

use ink_lang2 as ink;

#[ink::contract(
    env = ink_core::env2::DefaultSrmlTypes,
    version = "0.1.0",
)]
mod noop {
    #[ink(storage)]
    struct Noop {}

    impl Noop {
        #[ink(constructor)]
        fn new(&mut self) {}
    }
}

fn main() {}
