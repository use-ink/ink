use ink_storage::{
    traits::Packed,
    Lazy,
};

#[ink_lang::storage_item]
#[derive(Default)]
struct NonPacked {
    a: Lazy<u128>,
}

fn consume_packed<T: Packed>() {}

fn main() {
    let _ = consume_packed::<NonPacked>();
}
