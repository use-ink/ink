use ink_prelude::vec::Vec;
use ink_storage::StorageValue;

#[ink_lang::storage_item]
struct NonAtomic {
    a: StorageValue<u128>,
}

#[ink_lang::storage_item]
struct Contract {
    a: Vec<NonAtomic>,
}

fn main() {}
