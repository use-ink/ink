use ink_storage::traits::{
    ManualKey,
    StorageKeyHolder,
};

#[ink_lang::storage_item]
struct Contract<KEY: StorageKeyHolder = ManualKey<123>> {
    a: u16,
    b: u16,
    c: u16,
}

fn main() {
    assert_eq!(<Contract as StorageKeyHolder>::KEY, 123);
}
