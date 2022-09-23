use ink::storage::traits::Storable;
use ink::storage::traits::{
    ManualKey,
    StorageKey,
};

#[ink::storage_item(derive = false)]
#[derive(Default)]
struct Contract<KEY: StorageKey = ManualKey<123>> {
    a: u16,
    b: u64,
    c: u128,
}

// Disabling of deriving allow to implement the trait manually
impl<KEY: StorageKey> Storable for Contract<KEY> {
    fn encode<T: scale::Output + ?Sized>(&self, _dest: &mut T) {}

    fn decode<I: scale::Input>(_input: &mut I) -> Result<Self, scale::Error> {
        Ok(Self {
            a: Default::default(),
            b: Default::default(),
            c: Default::default(),
        })
    }
}

fn main() {
    let _: Result<Contract<ManualKey<123>>, _> = Storable::decode(&mut &[][..]);
}
