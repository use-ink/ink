use ink_prelude::collections::BTreeMap;
use ink_storage::StorageValue;

#[ink_lang::storage_item]
struct NonAtomic {
    a: StorageValue<u128>,
}

#[ink_lang::storage_item]
struct Contract {
    a: BTreeMap<u128, NonAtomic>,
}

fn main() {}
