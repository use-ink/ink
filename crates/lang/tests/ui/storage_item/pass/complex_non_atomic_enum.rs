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
enum Atomic {
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
enum NonAtomic<KEY: StorageKeyHolder = AutoKey> {
    #[default]
    None,
    A(Mapping<u128, Atomic>),
    B(StorageValue<u128>),
    C(StorageValue<Atomic>),
    D(StorageValue<Vec<Atomic>>),
    E((Mapping<String, Atomic>, StorageValue<Atomic>)),
    F {
        a: Mapping<String, Atomic>,
        b: (StorageValue<Atomic>, StorageValue<Atomic>),
    },
}

#[ink_lang::storage_item]
#[derive(Default)]
struct Contract {
    a: StorageValue<NonAtomic>,
}

fn main() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let mut contract = Contract::default();
        assert_eq!(contract.storage_key(), 0);

        // contract.a
        assert_eq!(
            contract.a.storage_key(),
            StorageKeyComposer::from_str("Contract::a")
        );
        assert_eq!(
            contract.a.get_or_default().storage_key(),
            StorageKeyComposer::from_str("Contract::a"),
        );

        contract.a.set(&NonAtomic::<_>::A(Default::default()));
        let variant = if let NonAtomic::<_>::A(variant) = contract.a.get() {
            variant
        } else {
            panic!("Wrong variant")
        };
        assert_eq!(
            variant.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("NonAtomic::A::0"),
                StorageKeyComposer::from_str("Contract::a")
            ),
        );

        contract.a.set(&NonAtomic::<_>::B(Default::default()));
        let variant = if let NonAtomic::<_>::B(variant) = contract.a.get() {
            variant
        } else {
            panic!("Wrong variant")
        };
        assert_eq!(
            variant.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("NonAtomic::B::0"),
                StorageKeyComposer::from_str("Contract::a")
            ),
        );

        contract.a.set(&NonAtomic::<_>::C(Default::default()));
        let variant = if let NonAtomic::<_>::C(variant) = contract.a.get() {
            variant
        } else {
            panic!("Wrong variant")
        };
        assert_eq!(
            variant.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("NonAtomic::C::0"),
                StorageKeyComposer::from_str("Contract::a")
            ),
        );

        contract.a.set(&NonAtomic::<_>::D(Default::default()));
        let variant = if let NonAtomic::<_>::D(variant) = contract.a.get() {
            variant
        } else {
            panic!("Wrong variant")
        };
        assert_eq!(
            variant.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("NonAtomic::D::0"),
                StorageKeyComposer::from_str("Contract::a")
            ),
        );

        contract.a.set(&NonAtomic::<_>::E(Default::default()));
        let variant = if let NonAtomic::<_>::E(variant) = contract.a.get() {
            variant
        } else {
            panic!("Wrong variant")
        };
        assert_eq!(
            variant.0.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("(A, B)::0"),
                StorageKeyComposer::concat(
                    StorageKeyComposer::from_str("NonAtomic::E::0"),
                    StorageKeyComposer::from_str("Contract::a")
                )
            ),
        );
        assert_eq!(
            variant.1.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("(A, B)::1"),
                StorageKeyComposer::concat(
                    StorageKeyComposer::from_str("NonAtomic::E::0"),
                    StorageKeyComposer::from_str("Contract::a")
                )
            ),
        );

        contract.a.set(&NonAtomic::<_>::F {
            a: Default::default(),
            b: Default::default(),
        });
        let variant = if let NonAtomic::<_>::F { a, b } = contract.a.get() {
            (a, b)
        } else {
            panic!("Wrong variant")
        };
        assert_eq!(
            variant.0.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("NonAtomic::F::a"),
                StorageKeyComposer::from_str("Contract::a")
            )
        );
        assert_eq!(
            variant.1 .0.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("(A, B)::0"),
                StorageKeyComposer::concat(
                    StorageKeyComposer::from_str("NonAtomic::F::b"),
                    StorageKeyComposer::from_str("Contract::a")
                )
            ),
        );
        assert_eq!(
            variant.1 .1.storage_key(),
            StorageKeyComposer::concat(
                StorageKeyComposer::from_str("(A, B)::1"),
                StorageKeyComposer::concat(
                    StorageKeyComposer::from_str("NonAtomic::F::b"),
                    StorageKeyComposer::from_str("Contract::a")
                )
            ),
        );
        Ok(())
    })
    .unwrap()
}
