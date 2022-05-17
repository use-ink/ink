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
enum Deep2 {
    #[default]
    None,
    A(u8),
    B(u16),
    C(u32),
    D(u64),
    E(u128),
    F(String),
    G {
        a: u8,
        b: String,
    },
    H((u16, u32)),
}

#[ink_lang::storage_item]
#[derive(Default)]
enum Deep1 {
    #[default]
    None,
    A(Deep2),
    B((Deep2, Deep2)),
    C(Vec<Deep2>),
    D(BTreeMap<Deep2, Deep2>),
    E(BTreeSet<Deep2>),
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
