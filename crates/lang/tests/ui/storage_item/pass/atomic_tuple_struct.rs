use ink_storage::traits::AtomicGuard;

#[ink_lang::storage_item]
#[derive(Default)]
struct Contract(String, u128);

fn main() {
    let _: &dyn AtomicGuard<true> = &Contract::default();
}
