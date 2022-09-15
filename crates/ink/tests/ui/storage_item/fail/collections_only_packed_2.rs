use ink_prelude::collections::BTreeMap;
use ink_storage::Lazy;

#[ink::storage_item]
struct NonPacked {
    a: Lazy<u128>,
}

#[ink::storage_item]
struct Contract {
    a: BTreeMap<u128, NonPacked>,
}

fn main() {}
