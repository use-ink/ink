use ink_prelude::collections::BTreeMap;
use ink_storage::Lazy;

#[ink_lang::storage_item]
struct NonAtomic {
    a: Lazy<u128>,
}

#[ink_lang::storage_item]
struct Contract {
    a: BTreeMap<u128, NonAtomic>,
}

fn main() {}
