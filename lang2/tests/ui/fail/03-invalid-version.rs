#![feature(proc_macro_hygiene)]

use ink_lang2 as ink;

#[ink::contract(
    env = DefaultSrmlTypes,
    version = "foo",
)]
mod invalid_version {
    #[ink(storage)]
    struct InvalidVersion {}
}

fn main() {}
