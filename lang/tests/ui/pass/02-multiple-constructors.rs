#![feature(proc_macro_hygiene)]

use ink_lang as ink;

#[ink::contract(
    env = DefaultSrmlTypes,
    version = [0, 1, 0],
)]
mod multiple_constructors {
    #[ink(storage)]
    struct MultipleConstructors {}

    impl MultipleConstructors {
        #[ink(constructor)]
        fn new_1(&mut self) {}

        #[ink(constructor)]
        fn new_2(&mut self) {}
    }
}

fn main() {}
