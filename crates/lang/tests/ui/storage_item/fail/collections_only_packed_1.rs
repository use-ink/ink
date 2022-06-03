use ink_prelude::vec::Vec;
use ink_storage::Lazy;

#[ink_lang::storage_item]
struct NonPacked {
    a: Lazy<u128>,
}

#[ink_lang::storage_item]
struct Contract {
    a: Vec<NonPacked>,
}

fn main() {}
