#![feature(proc_macro_hygiene)]

use ink_lang as ink;

#[ink::contract]
mod noop {
    #![ink(env = DefaultSrmlTypes)]

    #[ink(storage)]
    struct Noop {}

    impl Noop {
        #[ink(constructor)]
        fn new(&mut self) {}
    }
}

fn main() {}
