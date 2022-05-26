use ink_storage::traits::{
    KeyHolder,
    ManualKey,
};

#[ink_lang::storage_item]
struct Contract<KEY: KeyHolder = ManualKey<123>> {
    a: u16,
    b: u16,
    c: u16,
}

fn main() {
    assert_eq!(<Contract as KeyHolder>::KEY, 123);
}
