use ink_prelude::vec::Vec;
use ink_primitives::KeyComposer;
use ink_storage::{
    traits::{
        AutoKey,
        StorageKey,
    },
    Lazy,
    Mapping,
};

#[derive(Default, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
)]
enum Packed {
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
enum NonPacked<KEY: StorageKey = AutoKey> {
    #[default]
    None,
    A(Mapping<u128, Packed>),
    B(Lazy<u128>),
    C(Lazy<Packed>),
    D(Lazy<Vec<Packed>>),
    E(Mapping<String, Packed>),
    F {
        a: Mapping<String, Packed>,
    },
}

#[ink_lang::storage_item]
#[derive(Default)]
struct Contract {
    a: Lazy<NonPacked>,
}

fn main() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let mut contract = Contract::default();
        assert_eq!(contract.key(), 0);

        // contract.a
        assert_eq!(contract.a.key(), KeyComposer::from_str("Contract::a"));
        assert_eq!(
            contract.a.get_or_default().key(),
            KeyComposer::from_str("Contract::a"),
        );

        contract.a.set(&NonPacked::<_>::A(Default::default()));
        let variant = if let Some(NonPacked::<_>::A(variant)) = contract.a.get() {
            variant
        } else {
            panic!("Wrong variant")
        };
        assert_eq!(
            variant.key(),
            KeyComposer::concat(
                KeyComposer::from_str("NonPacked::A::0"),
                KeyComposer::from_str("Contract::a")
            ),
        );

        contract.a.set(&NonPacked::<_>::B(Default::default()));
        let variant = if let Some(NonPacked::<_>::B(variant)) = contract.a.get() {
            variant
        } else {
            panic!("Wrong variant")
        };
        assert_eq!(
            variant.key(),
            KeyComposer::concat(
                KeyComposer::from_str("NonPacked::B::0"),
                KeyComposer::from_str("Contract::a")
            ),
        );

        contract.a.set(&NonPacked::<_>::C(Default::default()));
        let variant = if let Some(NonPacked::<_>::C(variant)) = contract.a.get() {
            variant
        } else {
            panic!("Wrong variant")
        };
        assert_eq!(
            variant.key(),
            KeyComposer::concat(
                KeyComposer::from_str("NonPacked::C::0"),
                KeyComposer::from_str("Contract::a")
            ),
        );

        contract.a.set(&NonPacked::<_>::D(Default::default()));
        let variant = if let Some(NonPacked::<_>::D(variant)) = contract.a.get() {
            variant
        } else {
            panic!("Wrong variant")
        };
        assert_eq!(
            variant.key(),
            KeyComposer::concat(
                KeyComposer::from_str("NonPacked::D::0"),
                KeyComposer::from_str("Contract::a")
            ),
        );

        contract.a.set(&NonPacked::<_>::E(Default::default()));
        let variant = if let Some(NonPacked::<_>::E(variant)) = contract.a.get() {
            variant
        } else {
            panic!("Wrong variant")
        };
        assert_eq!(
            variant.key(),
            KeyComposer::concat(
                KeyComposer::from_str("NonPacked::E::0"),
                KeyComposer::from_str("Contract::a")
            )
        );

        contract.a.set(&NonPacked::<_>::F {
            a: Default::default(),
        });
        let variant = if let Some(NonPacked::<_>::F { a }) = contract.a.get() {
            a
        } else {
            panic!("Wrong variant")
        };
        assert_eq!(
            variant.key(),
            KeyComposer::concat(
                KeyComposer::from_str("NonPacked::F::a"),
                KeyComposer::from_str("Contract::a")
            )
        );
        Ok(())
    })
    .unwrap()
}
