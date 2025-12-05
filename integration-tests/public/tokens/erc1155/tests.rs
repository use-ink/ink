use crate::erc1155::*;
use crate::{Erc1155, Error};
use ink::{Address, U256};

fn set_sender(sender: Address) {
    ink::env::test::set_caller(sender);
}

fn default_accounts() -> ink::env::test::DefaultAccounts {
    ink::env::test::default_accounts()
}

fn alice() -> Address {
    default_accounts().alice
}

fn bob() -> Address {
    default_accounts().bob
}

fn charlie() -> Address {
    default_accounts().charlie
}

fn init_contract() -> Contract {
    set_sender(alice());
    let mut erc = Contract::new();
    // We can access internal storage directly for setup because the test module
    // is part of the crate hierarchy, though typically fields should be pub
    // or accessed via methods. In ink! storage structs are usually generated
    // with pub fields by the macro or we are checking behavior via public API.
    //
    // Note: In the refactor, we are using the public API where possible,
    // but here we are manually inserting into storage which requires visibility.
    // Ensure Contract fields are pub or accessible if this fails, but
    // usually #[ink::contract] makes storage fields private by default unless
    // explicitly pub. In the original code, they were inside the module.
    // Since we moved tests out, we might need to rely on public methods
    // like `create` or `mint` to set up state, OR make Contract fields pub.
    //
    // However, looking at the provided lib.rs, `balances` is NOT pub.
    // We should use `mint` or `create` to set up state instead of manipulating
    // private storage, which is a better testing practice anyway.

    // Let's use public API to replicate the setup:
    // Original:
    // erc.balances.insert((alice(), 1), &U256::from(10));
    // erc.balances.insert((alice(), 2), &U256::from(20));
    // erc.balances.insert((bob(), 1), &U256::from(10));

    // New way using public API:
    let _ = erc.create(10.into()); // Token ID 1 for Alice
    let _ = erc.create(20.into()); // Token ID 2 for Alice

    // Setup Bob's balance: Alice transfers to Bob, or we temporarily switch caller
    // Since create/mint assigns to caller.
    set_sender(bob());
    // Bob needs Token ID 1. But Token ID 1 is already created by Alice.
    // Bob can mint more if logic allows, or Alice transfers.
    // The `mint` function allows minting existing tokens.
    // But `mint` checks if token exists. Token 1 exists (nonce is incremented).
    let _ = erc.mint(1, 10.into()); // Bob mints 10 of Token ID 1

    // Reset sender to Alice
    set_sender(alice());

    erc
}

#[ink::test]
fn can_get_correct_balance_of() {
    let erc = init_contract();

    assert_eq!(erc.balance_of(alice(), 1), U256::from(10));
    assert_eq!(erc.balance_of(alice(), 2), U256::from(20));
    assert_eq!(erc.balance_of(alice(), 3), U256::zero());
    assert_eq!(erc.balance_of(bob(), 1), U256::from(10)); // Bob has 10 of ID 1
    assert_eq!(erc.balance_of(bob(), 2), U256::zero());
}

#[ink::test]
fn can_get_correct_batch_balance_of() {
    let erc = init_contract();

    assert_eq!(
        erc.balance_of_batch(vec![alice()], vec![1, 2, 3]),
        vec![U256::from(10), 20.into(), 0.into()]
    );
    // Modified expectation: Bob has 10 of Token 1 from init_contract()
    assert_eq!(
        erc.balance_of_batch(vec![alice(), bob()], vec![1]),
        vec![U256::from(10), 10.into()]
    );

    assert_eq!(
        erc.balance_of_batch(vec![alice(), bob(), charlie()], vec![1, 2]),
        vec![
            U256::from(10),
            20.into(),
            10.into(), // Bob has 10 of Token 1
            0.into(),
            0.into(),
            0.into()
        ]
    );
}

#[ink::test]
fn can_send_tokens_between_accounts() {
    let mut erc = init_contract();

    assert!(
        erc.safe_transfer_from(alice(), bob(), 1, 5.into(), vec![])
            .is_ok()
    );
    assert_eq!(erc.balance_of(alice(), 1), U256::from(5));
    // Bob started with 10, got 5 more
    assert_eq!(erc.balance_of(bob(), 1), U256::from(15));

    assert!(
        erc.safe_transfer_from(alice(), bob(), 2, 5.into(), vec![])
            .is_ok()
    );
    assert_eq!(erc.balance_of(alice(), 2), U256::from(15));
    assert_eq!(erc.balance_of(bob(), 2), U256::from(5));
}

#[ink::test]
fn sending_too_many_tokens_fails() {
    let mut erc = init_contract();
    let res = erc.safe_transfer_from(alice(), bob(), 1, 99.into(), vec![]);
    assert_eq!(res.unwrap_err(), Error::InsufficientU256);
}

#[ink::test]
fn sending_tokens_to_zero_address_fails() {
    let burn: Address = [0; 20].into();

    let mut erc = init_contract();
    let res = erc.safe_transfer_from(alice(), burn, 1, 10.into(), vec![]);
    assert_eq!(res.unwrap_err(), Error::ZeroAddressTransfer);
}

#[ink::test]
fn can_send_batch_tokens() {
    let mut erc = init_contract();
    assert!(
        erc.safe_batch_transfer_from(
            alice(),
            bob(),
            vec![1, 2],
            vec![U256::from(5), U256::from(10)],
            vec![]
        )
        .is_ok()
    );

    let balances = erc.balance_of_batch(vec![alice(), bob()], vec![1, 2]);
    assert_eq!(
        balances,
        // Alice: 10-5=5, 20-10=10
        // Bob: 10+5=15, 0+10=10
        vec![U256::from(5), 10.into(), 15.into(), 10.into()]
    );
}

#[ink::test]
fn rejects_batch_if_lengths_dont_match() {
    let mut erc = init_contract();
    let res = erc.safe_batch_transfer_from(
        alice(),
        bob(),
        vec![1, 2, 3],
        vec![U256::from(5)],
        vec![],
    );
    assert_eq!(res.unwrap_err(), Error::BatchTransferMismatch);
}

#[ink::test]
fn batch_transfers_fail_if_len_is_zero() {
    let mut erc = init_contract();
    let res =
        erc.safe_batch_transfer_from(alice(), bob(), vec![], vec![], vec![]);
    assert_eq!(res.unwrap_err(), Error::BatchTransferMismatch);
}

#[ink::test]
fn operator_can_send_tokens() {
    let mut erc = init_contract();

    let owner = alice();
    let operator = bob();

    set_sender(owner);
    assert!(erc.set_approval_for_all(operator, true).is_ok());

    set_sender(operator);
    assert!(
        erc.safe_transfer_from(owner, charlie(), 1, 5.into(), vec![])
            .is_ok()
    );
    assert_eq!(erc.balance_of(alice(), 1), U256::from(5));
    assert_eq!(erc.balance_of(charlie(), 1), U256::from(5));
}

#[ink::test]
fn approvals_work() {
    let mut erc = init_contract();
    let owner = alice();
    let operator = bob();
    let another_operator = charlie();

    // Note: All of these tests are from the context of the owner who is either
    // allowing or disallowing an operator to control their funds.
    set_sender(owner);
    assert!(!erc.is_approved_for_all(owner, operator));

    assert!(erc.set_approval_for_all(operator, true).is_ok());
    assert!(erc.is_approved_for_all(owner, operator));

    assert!(erc.set_approval_for_all(another_operator, true).is_ok());
    assert!(erc.is_approved_for_all(owner, another_operator));

    assert!(erc.set_approval_for_all(operator, false).is_ok());
    assert!(!erc.is_approved_for_all(owner, operator));
}

#[ink::test]
fn minting_tokens_works() {
    let mut erc = Contract::new();

    set_sender(alice());
    assert_eq!(erc.create(0.into()), 1);
    assert_eq!(erc.balance_of(alice(), 1), U256::zero());

    assert!(erc.mint(1, 123.into()).is_ok());
    assert_eq!(erc.balance_of(alice(), 1), U256::from(123));
}

#[ink::test]
fn minting_not_allowed_for_nonexistent_tokens() {
    let mut erc = Contract::new();

    let res = erc.mint(1, 123.into());
    assert_eq!(res.unwrap_err(), Error::UnexistentToken);
}