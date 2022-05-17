use ink_storage::{
    traits::AtomicGuard,
    StorageValue,
};

#[ink_lang::storage_item]
#[derive(Default)]
struct NonAtomic {
    a: StorageValue<u128>,
}

fn main() {
    let _: &dyn AtomicGuard<true> = &NonAtomic::default();
}
