use ink_prelude::{
    collections::{
        BTreeMap,
        BTreeSet,
    },
    vec::Vec,
};
use ink::storage::traits::Storable;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
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

#[derive(Default)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
enum Deep1 {
    #[default]
    None,
    A(Deep2),
    B((Deep2, Deep2)),
    C(Vec<Deep2>),
    D(BTreeMap<Deep2, Deep2>),
    E(BTreeSet<Deep2>),
}

#[derive(Default)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[ink::scale_derive(Encode, Decode)]
struct Contract {
    a: Deep1,
    b: Deep2,
    c: (Deep1, Deep2),
}

fn main() {
    let _: Result<Contract, _> = Storable::decode(&mut &[][..]);
}
