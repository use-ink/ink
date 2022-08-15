use ink_storage::{
    traits,
    traits::ManualKey,
};

#[ink_lang::storage_item]
struct Contract<KEY: traits::StorageKey = ManualKey<123>> {
    a: u16,
    b: u16,
    c: u16,
}

fn main() {
    assert_eq!(<Contract as traits::StorageKey>::KEY, 123);
}
