use ink::storage::traits::{
    ManualKey,
    StorageKey,
};

#[ink::storage_item]
struct Contract<KEY: StorageKey = ManualKey<123>> {
    a: u16,
    b: u16,
    c: u16,
}

fn main() {
    assert_eq!(<Contract as StorageKey>::KEY, 123);
}
