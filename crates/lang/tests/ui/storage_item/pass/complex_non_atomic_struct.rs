use ink_prelude::vec::Vec;
use ink_primitives::KeyComposer;
use ink_storage::{
    traits::{
        AutoKey,
        KeyHolder,
    },
    Lazy,
    Mapping,
};

#[ink_lang::packed]
#[derive(Default)]
struct Packed {
    a: u8,
    b: u16,
    c: u32,
    d: u64,
    e: u128,
    f: String,
}

#[ink_lang::non_packed]
#[derive(Default)]
struct NonPacked<KEY: KeyHolder = AutoKey> {
    a: Mapping<u128, Packed>,
    b: Lazy<u128>,
    c: Lazy<Packed>,
    d: Lazy<Vec<Packed>>,
}

#[ink_lang::non_packed]
#[derive(Default)]
struct Contract {
    a: Lazy<NonPacked>,
    b: Mapping<u128, Packed>,
    c: (Packed, Packed),
}

fn main() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let contract = Contract::default();
        assert_eq!(contract.key(), 0);

        // contract.b
        assert_eq!(contract.b.key(), KeyComposer::from_str("Contract::b"));

        // contract.c
        assert_eq!(
            contract.c.0.key(),
            KeyComposer::concat(
                KeyComposer::from_str("(A, B)::0"),
                KeyComposer::from_str("Contract::c")
            )
        );
        assert_eq!(
            contract.c.1.key(),
            KeyComposer::concat(
                KeyComposer::from_str("(A, B)::1"),
                KeyComposer::from_str("Contract::c")
            )
        );

        // contract.a
        assert_eq!(contract.a.key(), KeyComposer::from_str("Contract::a"));

        assert_eq!(
            contract.a.get_or_default().a.key(),
            KeyComposer::concat(
                KeyComposer::from_str("NonPacked::a"),
                KeyComposer::from_str("Contract::a")
            ),
        );

        assert_eq!(
            contract.a.get_or_default().b.key(),
            KeyComposer::concat(
                KeyComposer::from_str("NonPacked::b"),
                KeyComposer::from_str("Contract::a")
            ),
        );

        assert_eq!(
            contract.a.get_or_default().c.key(),
            KeyComposer::concat(
                KeyComposer::from_str("NonPacked::c"),
                KeyComposer::from_str("Contract::a")
            ),
        );

        assert_eq!(
            contract.a.get_or_default().d.key(),
            KeyComposer::concat(
                KeyComposer::from_str("NonPacked::d"),
                KeyComposer::from_str("Contract::a")
            ),
        );
        Ok(())
    })
    .unwrap()
}
