use ink::storage::traits::Storable;

#[derive(Default)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
#[ink::scale_derive(encode, decode, type_info)]
struct Contract(String, u128);

fn main() {
    let _: Result<Contract, _> = Storable::decode(&mut &[][..]);
}
