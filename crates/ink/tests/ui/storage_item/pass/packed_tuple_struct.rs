use ink::storage::traits::Storable;

#[derive(Default)]
#[ink::storage_item(packed)]
struct Contract(String, u128);

fn main() {
    let _: Result<Contract, _> = Storable::decode(&mut &[][..]);
}
