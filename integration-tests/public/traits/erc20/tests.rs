use super::erc20::*;
use ink::{
    env::hash::{
        Blake2x256,
        CryptoHash,
        HashOutput,
    },
    primitives::{
        Clear,
        Hash,
    },
    U256,
};

fn assert_transfer_event(
    event: &ink::env::test::EmittedEvent,
    expected_from: Option<Address>,
    expected_to: Option<Address>,
    expected_value: U256,
) {
    let decoded_event = <Transfer as ink::scale::Decode>::decode(&mut &event.data[..])
        .expect("encountered invalid contract event data buffer");
    let Transfer { from, to, value } = decoded_event;
    assert_eq!(from, expected_from, "encountered invalid Transfer.from");
    assert_eq!(to, expected_to, "encountered invalid Transfer.to");
    assert_eq!(value, expected_value, "encountered invalid Transfer.value");

    fn encoded_into_hash<T>(entity: T) -> Hash
    where
        T: ink::scale::Encode,
    {
        let mut result = Hash::CLEAR_HASH;
        let len_result = result.as_ref().len();
        let encoded = entity.encode();
        let len_encoded = encoded.len();
        if len_encoded <= len_result {
            result.as_mut()[..len_encoded].copy_from_slice(&encoded);
            return result
        }
        let mut hash_output = <<Blake2x256 as HashOutput>::Type as Default>::default();
        <Blake2x256 as CryptoHash>::hash(&encoded, &mut hash_output);
        let copy_len = core::cmp::min(hash_output.len(), len_result);
        result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
        result
    }

    let mut expected_topics = Vec::new();
    expected_topics.push(
        ink::blake2x256!("Transfer(Option<Address>,Option<Address>,U256)").into(),
    );
    if let Some(from) = expected_from {
        expected_topics.push(encoded_into_hash(from));
    } else {
        expected_topics.push(Hash::CLEAR_HASH);
    }
    if let Some(to) = expected_to {
        expected_topics.push(encoded_into_hash(to));
    } else {
        expected_topics.push(Hash::CLEAR_HASH);
    }
    expected_topics.push(encoded_into_hash(value));

    for (n, (actual_topic, expected_topic)) in
        event.topics.iter().zip(expected_topics).enumerate()
    {
        let topic = <Hash as ink::scale::Decode>::decode(&mut &actual_topic[..])
            .expect("encountered invalid topic encoding");
        assert_eq!(topic, expected_topic, "encountered invalid topic at {n}");
    }
}

/// The default constructor does its job.
#[ink::test]
fn new_works() {
    // Constructor works.
    set_caller(Address::from([0x01; 20]));
    let initial_supply = 100.into();
    let erc20 = Erc20::new(initial_supply);

    // The `BaseErc20` trait has indeed been implemented.
    assert_eq!(<Erc20 as BaseErc20>::total_supply(&erc20), initial_supply);

    // Transfer event triggered during initial construction.
    let emitted_events = ink::env::test::recorded_events();
    assert_eq!(1, emitted_events.len());

    assert_transfer_event(
        &emitted_events[0],
        None,
        Some(Address::from([0x01; 20])),
        100.into(),
    );
}

/// The total supply was applied.
#[ink::test]
fn total_supply_works() {
    // Constructor works.
    set_caller(Address::from([0x01; 20]));
    let initial_supply = 100.into();
    let erc20 = Erc20::new(initial_supply);
    // Transfer event triggered during initial construction.
    let emitted_events = ink::env::test::recorded_events();
    assert_transfer_event(
        &emitted_events[0],
        None,
        Some(Address::from([0x01; 20])),
        100.into(),
    );
    // Get the token total supply.
    assert_eq!(erc20.total_supply(), 100.into());
}

/// Get the actual balance of an account.
#[ink::test]
fn balance_of_works() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);

    // Constructor works
    let initial_supply = 100.into();
    let erc20 = Erc20::new(initial_supply);
    // Transfer event triggered during initial construction
    let emitted_events = ink::env::test::recorded_events();
    assert_transfer_event(
        &emitted_events[0],
        None,
        Some(accounts.alice),
        100.into(),
    );
    // Alice owns all the tokens on contract instantiation
    assert_eq!(erc20.balance_of(accounts.alice), 100.into());
    // Bob does not owns tokens
    assert_eq!(erc20.balance_of(accounts.bob), U256::zero());
}

#[ink::test]
fn transfer_works() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);

    // Constructor works.
    let initial_supply = 100.into();
    let mut erc20 = Erc20::new(initial_supply);
    // Transfer event triggered during initial construction.
    assert_eq!(erc20.balance_of(accounts.bob), U256::zero());
    // Alice transfers 10 tokens to Bob.
    assert_eq!(erc20.transfer(accounts.bob, U256::from(10)), Ok(()));
    // Bob owns 10 tokens.
    assert_eq!(erc20.balance_of(accounts.bob), U256::from(10));

    let emitted_events = ink::env::test::recorded_events();
    assert_eq!(emitted_events.len(), 2);
    // Check first transfer event related to ERC-20 instantiation.
    assert_transfer_event(
        &emitted_events[0],
        None,
        Some(accounts.alice),
        100.into(),
    );
    // Check the second transfer event relating to the actual transfer.
    assert_transfer_event(
        &emitted_events[1],
        Some(accounts.alice),
        Some(accounts.bob),
        10.into(),
    );
}

#[ink::test]
fn invalid_transfer_should_fail() {
    // Constructor works.
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);

    let initial_supply = 100.into();
    let mut erc20 = Erc20::new(initial_supply);

    assert_eq!(erc20.balance_of(accounts.bob), U256::zero());
    // Set Bob as caller
    set_caller(accounts.bob);

    // Bob fails to transfer 10 tokens to Eve.
    assert_eq!(
        erc20.transfer(accounts.eve, 10.into()),
        Err(Error::InsufficientBalance)
    );
    // Alice owns all the tokens.
    assert_eq!(erc20.balance_of(accounts.alice), 100.into());
    assert_eq!(erc20.balance_of(accounts.bob), U256::zero());
    assert_eq!(erc20.balance_of(accounts.eve), U256::zero());

    // Transfer event triggered during initial construction.
    let emitted_events = ink::env::test::recorded_events();
    assert_eq!(emitted_events.len(), 1);
    assert_transfer_event(
        &emitted_events[0],
        None,
        Some(accounts.alice),
        100.into(),
    );
}

#[ink::test]
fn transfer_from_works() {
    // Constructor works.
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);

    let initial_supply = 100.into();
    let mut erc20 = Erc20::new(initial_supply);

    // Transfer event triggered during initial construction.
    let accounts = ink::env::test::default_accounts();

    // Bob fails to transfer tokens owned by Alice.
    assert_eq!(
        erc20.transfer_from(accounts.alice, accounts.eve, 10.into()),
        Err(Error::InsufficientAllowance)
    );
    // Alice approves Bob for token transfers on her behalf.
    assert_eq!(erc20.approve(accounts.bob, U256::from(10)), Ok(()));

    // The approve event takes place.
    assert_eq!(ink::env::test::recorded_events().len(), 2);

    // Set Bob as caller.
    set_caller(accounts.bob);

    // Bob transfers tokens from Alice to Eve.
    assert_eq!(
        erc20.transfer_from(accounts.alice, accounts.eve, 10.into()),
        Ok(())
    );
    // Eve owns tokens.
    assert_eq!(erc20.balance_of(accounts.eve), U256::from(10));

    // Check all transfer events that happened during the previous calls:
    let emitted_events = ink::env::test::recorded_events();
    assert_eq!(emitted_events.len(), 3);
    assert_transfer_event(
        &emitted_events[0],
        None,
        Some(accounts.alice),
        100.into(),
    );
    // The second event `emitted_events[1]` is an Approve event that we skip
    // checking.
    assert_transfer_event(
        &emitted_events[2],
        Some(accounts.alice),
        Some(accounts.eve),
        10.into(),
    );
}

#[ink::test]
fn allowance_must_not_change_on_failed_transfer() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);
    let initial_supply = 100.into();
    let mut erc20 = Erc20::new(initial_supply);

    // Alice approves Bob for token transfers on her behalf.
    let alice_balance = erc20.balance_of(accounts.alice);
    let initial_allowance = alice_balance + U256::from(2);
    assert_eq!(erc20.approve(accounts.bob, initial_allowance), Ok(()));

    // Set Bob as caller.
    set_caller(accounts.bob);

    // Bob tries to transfer tokens from Alice to Eve.
    let emitted_events_before = ink::env::test::recorded_events();
    assert_eq!(
        erc20.transfer_from(
            accounts.alice,
            accounts.eve,
            alice_balance + U256::from(1)
        ),
        Err(Error::InsufficientBalance)
    );
    // Allowance must have stayed the same
    assert_eq!(
        erc20.allowance(accounts.alice, accounts.bob),
        initial_allowance
    );
    // No more events must have been emitted
    let emitted_events_after = ink::env::test::recorded_events();
    assert_eq!(emitted_events_before.len(), emitted_events_after.len());
}

fn set_caller(sender: Address) {
    ink::env::test::set_caller(sender);
}