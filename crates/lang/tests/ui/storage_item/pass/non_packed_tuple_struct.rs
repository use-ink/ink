use ink_primitives::KeyComposer;
use ink_storage::{
    traits::StorageKey,
    Lazy,
    Mapping,
};

#[ink_lang::storage_item]
#[derive(Default)]
struct Contract(Mapping<u128, String>, Lazy<u128>);

fn main() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let contract = Contract::default();
        assert_eq!(contract.key(), 0);

        assert_eq!(contract.0.key(), KeyComposer::from_str("Contract::0"));
        assert_eq!(contract.1.key(), KeyComposer::from_str("Contract::1"));
        Ok(())
    })
    .unwrap()
}
