#![feature(proc_macro_hygiene)]

use ink_lang2 as ink;

#[ink::contract(
    env = DefaultSrmlTypes,
    version = "0.1.0",
)]
mod noop {
    #[ink(storage)]
    struct Noop {}

    impl Noop {
        #[ink(message)]
        fn noop(&self) {}
    }
}

fn main() {}
