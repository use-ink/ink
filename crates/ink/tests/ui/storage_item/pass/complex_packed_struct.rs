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
struct Deep2 {
    a: u8,
    b: u16,
    c: u32,
    d: u64,
    e: u128,
    f: String,
}

#[derive(Default)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
struct Deep1 {
    a: Deep2,
    b: (Deep2, Deep2),
    c: Vec<Deep2>,
    d: BTreeMap<Deep2, Deep2>,
    e: BTreeSet<Deep2>,
}

#[derive(Default)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
struct Contract {
    a: Deep1,
    b: Deep2,
    c: (Deep1, Deep2),
}

fn main() {
    let _: Result<Contract, _> = Storable::decode(&mut &[][..]);
}
