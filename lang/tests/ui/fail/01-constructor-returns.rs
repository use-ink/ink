#![feature(proc_macro_hygiene)]

use ink_lang as ink;

#[ink::contract(
    env = DefaultSrmlTypes,
    version = [0, 1, 0],
)]
mod noop {
    #[ink(storage)]
    struct Noop {}

    #[ink(impl)]
    impl Noop {
        #[ink(constructor)]
        fn new(&mut self) -> Self {}
    }
}

fn main() {}
