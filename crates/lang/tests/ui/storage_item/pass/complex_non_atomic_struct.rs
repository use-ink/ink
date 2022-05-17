use ink_prelude::vec::Vec;
use ink_primitives::StorageKeyComposer;
use ink_storage::{
    traits::{
        AutoKey,
        StorageKeyHolder,
    },
    Mapping,
    StorageValue,
};

#[ink_lang::storage_item]
#[derive(Default)]
struct Atomic {
    a: u8,
    b: u16,
    c: u32,
    d: u64,
    e: u128,
    f: String,
}

#[ink_lang::storage_item]
#[derive(Default)]
struct NonAtomic<KEY: StorageKeyHolder = AutoKey> {
    a: Mapping<u128, Atomic>,
    b: StorageValue<u128>,
    c: StorageValue<Atomic>,
    d: StorageValue<Vec<Atomic>>,
}

#[ink_lang::storage_item]
#[derive(Default)]
struct Contract {
    a: StorageValue<NonAtomic>,
    b: Mapping<u128, Atomic>,
    c: (StorageValue<NonAtomic>, StorageValue<Atomic>),
}

fn main() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let contract = Contract::default();
        assert_eq!(contract.storage_key(), 0);

        // contract.b
        assert_eq!(
            contract.b.storage_key(),
            StorageKeyComposer::from_str("Contract::b")
        );

        // contract.c
        assert_eq!(
            contract.c.0.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("(A, B)::0"),
                StorageKeyComposer::from_str("Contract::c")
            )
        );
        assert_eq!(
            contract.c.1.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("(A, B)::1"),
                StorageKeyComposer::from_str("Contract::c")
            )
        );

        // contract.a
        assert_eq!(
            contract.a.storage_key(),
            StorageKeyComposer::from_str("Contract::a")
        );

        assert_eq!(
            contract.a.get_or_default().a.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("NonAtomic::a"),
                StorageKeyComposer::from_str("Contract::a")
            ),
        );

        assert_eq!(
            contract.a.get_or_default().b.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("NonAtomic::b"),
                StorageKeyComposer::from_str("Contract::a")
            ),
        );

        assert_eq!(
            contract.a.get_or_default().c.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("NonAtomic::c"),
                StorageKeyComposer::from_str("Contract::a")
            ),
        );

        assert_eq!(
            contract.a.get_or_default().d.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("NonAtomic::d"),
                StorageKeyComposer::from_str("Contract::a")
            ),
        );
        Ok(())
    })
    .unwrap()
}
