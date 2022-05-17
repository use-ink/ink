use ink_primitives::StorageKeyComposer;
use ink_storage::{
    traits::StorageKeyHolder,
    Mapping,
    StorageValue,
};

#[ink_lang::storage_item]
#[derive(Default)]
struct Contract(Mapping<u128, String>, StorageValue<u128>);

fn main() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let contract = Contract::default();
        assert_eq!(contract.storage_key(), 0);

        assert_eq!(
            contract.0.storage_key(),
            StorageKeyComposer::from_str("Contract::0")
        );
        assert_eq!(
            contract.1.storage_key(),
            StorageKeyComposer::from_str("Contract::1")
        );
        Ok(())
    })
    .unwrap()
}
