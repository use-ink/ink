use ink_storage::traits::{
    AtomicGuard,
    KeyHolder,
    ManualKey,
};

#[ink_lang::storage_item(derive = false)]
#[derive(Default)]
struct Contract<KEY: KeyHolder = ManualKey<123>> {
    a: u16,
    b: u64,
    c: u128,
}

// Disabling of deriving allow to implement the trait manually
impl<KEY: KeyHolder> AtomicGuard<true> for Contract<KEY> {}

fn main() {
    let _: &dyn AtomicGuard<true> = &Contract::<ManualKey<123>>::default();
}
