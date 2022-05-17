use ink_prelude::{
    collections::{
        BTreeMap,
        BTreeSet,
    },
    vec::Vec,
};
use ink_storage::traits::AtomicGuard;

#[ink_lang::storage_item]
#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
struct Deep2 {
    a: u8,
    b: u16,
    c: u32,
    d: u64,
    e: u128,
    f: String,
}

#[ink_lang::storage_item]
#[derive(Default)]
struct Deep1 {
    a: Deep2,
    b: (Deep2, Deep2),
    c: Vec<Deep2>,
    d: BTreeMap<Deep2, Deep2>,
    e: BTreeSet<Deep2>,
}

#[ink_lang::storage_item]
#[derive(Default)]
struct Contract {
    a: Deep1,
    b: Deep2,
    c: (Deep1, Deep2),
}

fn main() {
    let _: &dyn AtomicGuard<true> = &Contract::default();
}
