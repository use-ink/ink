use ink::storage::traits::Storable;

#[derive(Default, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
struct Contract(String, u128);

fn main() {
    let _: Result<Contract, _> = Storable::decode(&mut &[][..]);
}
