use crate::erc20::*;
use ink::primitives::{
    Address,
    Clear,
    Hash,
};
use ink::U256;

// --- Helper Functions for Unit Tests ---

fn set_caller(sender: Address) {
    ink::env::test::set_caller(sender);
}

fn encoded_into_hash<T>(entity: T) -> Hash
where
    T: ink::scale::Encode,
{
    use ink::{
        env::hash::{
            Blake2x256,
            CryptoHash,
            HashOutput,
        },
        primitives::Clear,
    };

    let mut result = Hash::CLEAR_HASH;
    let len_result = result.as_ref().len();
    let encoded = entity.encode();
    let len_encoded = encoded.len();
    if len_encoded <= len_result {
        result.as_mut()[..len_encoded].copy_from_slice(&encoded);
        return result
    }
    let mut hash_output =
        <<Blake2x256 as HashOutput>::Type as Default>::default();
    <Blake2x256 as CryptoHash>::hash(&encoded, &mut hash_output);
    let copy_len = core::cmp::min(hash_output.len(), len_result);
    result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
    result
}

fn assert_transfer_event(
    event: &ink::env::test::EmittedEvent,
    expected_from: Option<Address>,
    expected_to: Option<Address>,
    expected_value: U256,
) {
    let decoded_event =
        <Transfer as ink::scale::Decode>::decode(&mut &event.data[..])
            .expect("encountered invalid contract event data buffer");
    let Transfer { from, to, value } = decoded_event;
    assert_eq!(from, expected_from, "encountered invalid Transfer.from");
    assert_eq!(to, expected_to, "encountered invalid Transfer.to");
    assert_eq!(value, expected_value, "encountered invalid Transfer.value");

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

    let topics = event.topics.clone();
    for (n, (actual_topic, expected_topic)) in
        topics.iter().zip(expected_topics).enumerate()
    {
        let mut topic_hash = Hash::CLEAR_HASH;
        let len = actual_topic.len();
        topic_hash.as_mut()[0..len].copy_from_slice(&actual_topic[0..len]);

        assert_eq!(
            topic_hash, expected_topic,
            "encountered invalid topic at {n}"
        );
    }
}

// --- Unit Tests ---

/// The default constructor does its job.
#[ink::test]
fn new_works() {
    // Constructor works.
    set_caller(Address::from([0x01; 20]));
    let _erc20 = Erc20::new(100.into());

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
    let erc20 = Erc20::new(100.into());
    // Transfer event triggered during initial construction.
    let emitted_events = ink::env::test::recorded_events();
    assert_transfer_event(
        &emitted_events[0],
        None,
        Some(Address::from([0x01; 20])),
        100.into(),
    );
    // Get the token total supply.
    assert_eq!(erc20.total_supply(), U256::from(100));
}

/// Get the actual balance of an account.
#[ink::test]
fn balance_of_works() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);

    // Constructor works
    let erc20 = Erc20::new(100.into());
    // Transfer event triggered during initial construction
    let emitted_events = ink::env::test::recorded_events();
    assert_transfer_event(
        &emitted_events[0],
        None,
        Some(accounts.alice),
        100.into(),
    );
    let accounts = ink::env::test::default_accounts();
    // Alice owns all the tokens on contract instantiation
    assert_eq!(erc20.balance_of(accounts.alice), U256::from(100));
    // Bob does not owns tokens
    assert_eq!(erc20.balance_of(accounts.bob), U256::zero());
}

#[ink::test]
fn transfer_works() {
    let accounts = ink::env::test::default_accounts();
    set_caller(accounts.alice);

    // Constructor works.
    let mut erc20 = Erc20::new(100.into());
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

    // Set the contract as callee and Bob as caller.
    let contract = ink::env::address();
    ink::env::test::set_callee(contract);
    set_caller(accounts.bob);

    // Bob fails to transfer 10 tokens to Eve.
    assert_eq!(
        erc20.transfer(accounts.eve, 10.into()),
        Err(Error::InsufficientBalance)
    );
    // Alice owns all the tokens.
    assert_eq!(erc20.balance_of(accounts.alice), U256::from(100));
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

    let mut erc20 = Erc20::new(100.into());

    // Bob fails to transfer tokens owned by Alice.
    assert_eq!(
        erc20.transfer_from(accounts.alice, accounts.eve, 10.into()),
        Err(Error::InsufficientAllowance)
    );
    // Alice approves Bob for token transfers on her behalf.
    assert_eq!(erc20.approve(accounts.bob, 10.into()), Ok(()));

    // The approve event takes place.
    assert_eq!(ink::env::test::recorded_events().len(), 2);

    // Set the contract as callee and Bob as caller.
    let contract = ink::env::address();
    ink::env::test::set_callee(contract);
    ink::env::test::set_caller(accounts.bob);

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
    let mut erc20 = Erc20::new(100.into());

    // Alice approves Bob for token transfers on her behalf.
    let alice_balance = erc20.balance_of(accounts.alice);
    let initial_allowance = alice_balance + 2;
    assert_eq!(erc20.approve(accounts.bob, initial_allowance), Ok(()));

    // Get contract address.
    let callee = ink::env::address();
    ink::env::test::set_callee(callee);
    ink::env::test::set_caller(accounts.bob);

    // Bob tries to transfer tokens from Alice to Eve.
    let emitted_events_before = ink::env::test::recorded_events().len();
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
    assert_eq!(
        emitted_events_before,
        ink::env::test::recorded_events().len()
    )
}

// --- E2E Tests ---

#[cfg(feature = "e2e-tests")]
mod e2e_tests {
    use super::*;
    use ink_e2e::ContractsBackend;

    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn e2e_transfer(mut client: Client) -> E2EResult<()> {
        // given
        let total_supply = U256::from(1_000_000_000);
        let mut constructor = Erc20Ref::new(total_supply);
        let erc20 = client
            .instantiate("erc20", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder = erc20.call_builder::<Erc20>();

        // when
        let total_supply_msg = call_builder.total_supply();
        let total_supply_res = client
            .call(&ink_e2e::bob(), &total_supply_msg)
            .dry_run()
            .await?;

        let bob_account = ink_e2e::address::<ink::env::DefaultEnvironment>(
            ink_e2e::Sr25519Keyring::Bob,
        );
        let transfer_to_bob = U256::from(500_000_000);
        let transfer = call_builder.transfer(bob_account, transfer_to_bob);
        let _transfer_res = client
            .call(&ink_e2e::alice(), &transfer)
            .submit()
            .await
            .expect("transfer failed");

        let balance_of = call_builder.balance_of(bob_account);
        let balance_of_res = client
            .call(&ink_e2e::alice(), &balance_of)
            .dry_run()
            .await?;

        // then
        assert_eq!(
            total_supply,
            total_supply_res.return_value(),
            "total_supply"
        );
        assert_eq!(transfer_to_bob, balance_of_res.return_value(), "balance_of");

        Ok(())
    }

    #[ink_e2e::test]
    async fn e2e_allowances(mut client: Client) -> E2EResult<()> {
        // given
        let total_supply = U256::from(1_000_000_000);
        let mut constructor = Erc20Ref::new(total_supply);
        let erc20 = client
            .instantiate("erc20", &ink_e2e::bob(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder = erc20.call_builder::<Erc20>();

        // when

        let bob_account = ink_e2e::address::<ink::env::DefaultEnvironment>(
            ink_e2e::Sr25519Keyring::Bob,
        );
        let charlie_account = ink_e2e::address::<ink::env::DefaultEnvironment>(
            ink_e2e::Sr25519Keyring::Charlie,
        );

        let amount = U256::from(500_000_000);
        // tx
        let transfer_from =
            call_builder.transfer_from(bob_account, charlie_account, amount);
        let transfer_from_result = client
            .call(&ink_e2e::charlie(), &transfer_from)
            .submit()
            .await;

        assert!(
            transfer_from_result.is_err(),
            "unapproved transfer_from should fail"
        );

        // Bob approves Charlie to transfer up to amount on his behalf
        let approved_value = U256::from(1_000);
        let approve_call = call_builder.approve(charlie_account, approved_value);
        client
            .call(&ink_e2e::bob(), &approve_call)
            .submit()
            .await
            .expect("approve failed");

        // `transfer_from` the approved amount
        let transfer_from =
            call_builder.transfer_from(bob_account, charlie_account, approved_value);
        let transfer_from_result = client
            .call(&ink_e2e::charlie(), &transfer_from)
            .submit()
            .await;
        assert!(
            transfer_from_result.is_ok(),
            "approved transfer_from should succeed"
        );

        let balance_of = call_builder.balance_of(bob_account);
        let balance_of_res = client
            .call(&ink_e2e::alice(), &balance_of)
            .dry_run()
            .await?;

        // `transfer_from` again, this time exceeding the approved amount
        let transfer_from =
            call_builder.transfer_from(bob_account, charlie_account, 1.into());
        let transfer_from_result = client
            .call(&ink_e2e::charlie(), &transfer_from)
            .submit()
            .await;
        assert!(
            transfer_from_result.is_err(),
            "transfer_from exceeding the approved amount should fail"
        );

        assert_eq!(
            total_supply - approved_value,
            balance_of_res.return_value(),
            "balance_of"
        );

        Ok(())
    }
}