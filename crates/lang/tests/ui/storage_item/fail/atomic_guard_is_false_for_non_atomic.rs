use ink_storage::{
    traits::AtomicGuard,
    Lazy,
};

#[ink_lang::storage_item]
#[derive(Default)]
struct NonAtomic {
    a: Lazy<u128>,
}

fn main() {
    let _: &dyn AtomicGuard<true> = &NonAtomic::default();
}
