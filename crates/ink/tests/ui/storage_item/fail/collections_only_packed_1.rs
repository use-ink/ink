use ink_prelude::vec::Vec;
use ink_storage::Lazy;

#[ink::storage_item]
struct NonPacked {
    a: Lazy<u128>,
}

#[ink::storage_item]
struct Contract {
    a: Vec<NonPacked>,
}

fn main() {}
