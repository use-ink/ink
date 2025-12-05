use super::payment_channel::*;
use ink::U256;
use ink::primitives::AccountId as Address;

use hex_literal;
use sp_core::{
    Encode,
    Pair,
};

fn default_accounts() -> ink::env::test::DefaultAccounts {
    ink::env::test::default_accounts()
}

fn set_next_caller(caller: Address) {
    ink::env::test::set_caller(caller);
}

fn set_contract_balance(addr: Address, balance: U256) {
    ink::env::test::set_contract_balance(addr, balance);
}

fn get_contract_balance(addr: Address) -> U256 {
    ink::env::test::get_contract_balance::<ink::env::DefaultEnvironment>(addr)
        .expect("Cannot get contract balance")
}

fn advance_block() {
    ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
}

fn get_current_time() -> u64 {
    let since_the_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
        + since_the_epoch.subsec_nanos() as u64 / 1_000_000_000
}

fn get_dan() -> Address {
    // Use Dan's seed
    // `subkey inspect //Dan --scheme Ecdsa --output-type json | jq .secretSeed`
    let seed = hex_literal::hex!(
        "c31fa562972de437802e0df146b16146349590b444db41f7e3eb9deedeee6f64"
    );
    let pair = sp_core::ecdsa::Pair::from_seed(&seed);
    let pub_key = pair.public();
    let compressed_pub_key: [u8; 33] = pub_key.encode()[..]
        .try_into()
        .expect("slice with incorrect length");
    let mut account_id = [0; 32];
    <ink::env::hash::Blake2x256 as ink::env::hash::CryptoHash>::hash(
        &compressed_pub_key,
        &mut account_id,
    );
    ink::primitives::AccountIdMapper::to_address(&account_id)
}

fn contract_id() -> Address {
    let accounts = default_accounts();
    let contract_id = accounts.charlie;
    ink::env::test::set_callee(contract_id);
    contract_id
}

fn sign(contract_id: Address, amount: U256) -> [u8; 65] {
    let encodable = (contract_id, amount);
    let mut hash =
        <ink::env::hash::Sha2x256 as ink::env::hash::HashOutput>::Type::default(); // 256-bit buffer
    ink::env::hash_encoded::<ink::env::hash::Sha2x256, _>(&encodable, &mut hash);

    // Use Dan's seed
    // `subkey inspect //Dan --scheme Ecdsa --output-type json | jq .secretSeed`
    let seed = hex_literal::hex!(
        "c31fa562972de437802e0df146b16146349590b444db41f7e3eb9deedeee6f64"
    );
    let pair = sp_core::ecdsa::Pair::from_seed(&seed);

    let signature = pair.sign_prehashed(&hash);
    signature.0
}

#[ink::test]
fn test_deposit() {
    // given
    let accounts = default_accounts();
    let initial_balance = 10_000.into();
    let close_duration = 360_000;
    let mock_deposit_value = 1_000.into();
    set_contract_balance(accounts.alice, initial_balance);
    set_contract_balance(accounts.bob, initial_balance);

    // when
    // Push the new execution context with Alice as the caller and
    // the `mock_deposit_value` as the value deposited.
    // Note: Currently there is no way to transfer funds to the contract.
    set_next_caller(accounts.alice);
    let payment_channel = PaymentChannel::new(accounts.bob, close_duration);
    let contract_id = contract_id();
    set_contract_balance(contract_id, mock_deposit_value);

    // then
    assert_eq!(payment_channel.get_balance(), mock_deposit_value);
}

#[ink::test]
fn test_close() {
    // given
    let accounts = default_accounts();
    let dan = get_dan();
    let close_duration = 360_000;
    let mock_deposit_value = 1_000.into();
    let amount = 500.into();
    let initial_balance = 10_000.into();
    set_contract_balance(accounts.alice, initial_balance);
    set_contract_balance(dan, initial_balance);

    // when
    set_next_caller(accounts.alice);
    let mut payment_channel = PaymentChannel::new(dan, close_duration);
    let contract_id = contract_id();
    set_contract_balance(contract_id, mock_deposit_value);
    set_next_caller(dan);
    let signature = sign(contract_id, amount);

    // then
    let should_close = move || payment_channel.close(amount, signature).unwrap();
    ink::env::test::assert_contract_termination::<ink::env::DefaultEnvironment, _>(
        should_close,
        accounts.alice,
        amount,
    );
    assert_eq!(get_contract_balance(dan), initial_balance + amount);
}

#[ink::test]
fn close_fails_invalid_signature() {
    // given
    let accounts = default_accounts();
    let dan = get_dan();
    let mock_deposit_value = 1_000.into();
    let close_duration = 360_000;
    let amount = 400.into();
    let unexpected_amount = amount + U256::from(1);
    let initial_balance = 10_000.into();
    set_contract_balance(accounts.alice, initial_balance);
    set_contract_balance(dan, initial_balance);

    // when
    set_next_caller(accounts.alice);
    let mut payment_channel = PaymentChannel::new(dan, close_duration);
    let contract_id = contract_id();
    set_contract_balance(contract_id, mock_deposit_value);
    set_next_caller(dan);
    let signature = sign(contract_id, amount);

    // then
    let res = payment_channel.close_inner(unexpected_amount, signature);
    assert!(res.is_err(), "Expected an error, got {res:?} instead.");
    assert_eq!(res.unwrap_err(), Error::InvalidSignature,);
}

#[ink::test]
fn test_withdraw() {
    // given
    let accounts = default_accounts();
    let dan = get_dan();
    let initial_balance = 10_000.into();
    let mock_deposit_value = 1_000.into();
    let close_duration = 360_000;
    let amount = 500.into();
    set_contract_balance(accounts.alice, initial_balance);
    set_contract_balance(dan, initial_balance);

    // when
    set_next_caller(accounts.alice);
    let mut payment_channel = PaymentChannel::new(dan, close_duration);
    let contract_id = contract_id();
    set_contract_balance(contract_id, mock_deposit_value);

    set_next_caller(dan);
    let signature = sign(contract_id, amount);
    payment_channel
        .withdraw(amount, signature)
        .expect("withdraw failed");

    // then
    assert_eq!(payment_channel.get_balance(), amount);
    assert_eq!(get_contract_balance(dan), initial_balance + amount);
}

#[ink::test]
fn withdraw_fails_invalid_signature() {
    // given
    let accounts = default_accounts();
    let dan = get_dan();
    let initial_balance = 10_000.into();
    let close_duration = 360_000;
    let amount = 400.into();
    let unexpected_amount = amount + U256::from(1);
    let mock_deposit_value = 1_000.into();
    set_contract_balance(accounts.alice, initial_balance);
    set_contract_balance(dan, initial_balance);

    // when
    set_next_caller(accounts.alice);
    let mut payment_channel = PaymentChannel::new(dan, close_duration);
    let contract_id = contract_id();
    set_contract_balance(contract_id, mock_deposit_value);
    set_next_caller(dan);
    let signature = sign(contract_id, amount);

    // then
    let res = payment_channel.withdraw(unexpected_amount, signature);
    assert!(res.is_err(), "Expected an error, got {res:?} instead.");
    assert_eq!(res.unwrap_err(), Error::InvalidSignature,);
}

#[ink::test]
fn test_start_sender_close() {
    // given
    let accounts = default_accounts();
    let initial_balance = 10_000.into();
    let mock_deposit_value = 1_000.into();
    let close_duration = 1;
    set_contract_balance(accounts.alice, initial_balance);
    set_contract_balance(accounts.bob, initial_balance);

    // when
    set_next_caller(accounts.alice);
    let mut payment_channel = PaymentChannel::new(accounts.bob, close_duration);
    let contract_id = contract_id();
    set_contract_balance(contract_id, mock_deposit_value);

    payment_channel
        .start_sender_close()
        .expect("start_sender_close failed");
    advance_block();

    // then
    let now = get_current_time();
    assert!(now > payment_channel.get_expiration().unwrap());
}

#[ink::test]
fn test_claim_timeout() {
    // given
    let accounts = default_accounts();
    let initial_balance = 10_000.into();
    let close_duration = 1;
    let mock_deposit_value = 1_000.into();
    set_contract_balance(accounts.alice, initial_balance);
    set_contract_balance(accounts.bob, initial_balance);

    // when
    set_next_caller(accounts.alice);
    let contract_id = contract_id();
    let mut payment_channel = PaymentChannel::new(accounts.bob, close_duration);
    set_contract_balance(contract_id, mock_deposit_value);

    payment_channel
        .start_sender_close()
        .expect("start_sender_close failed");
    advance_block();

    // then
    let should_close = move || payment_channel.claim_timeout().unwrap();
    ink::env::test::assert_contract_termination::<ink::env::DefaultEnvironment, _>(
        should_close,
        accounts.alice,
        mock_deposit_value,
    );
    assert_eq!(
        get_contract_balance(accounts.alice),
        initial_balance + mock_deposit_value
    );
}

#[ink::test]
fn test_getters() {
    // given
    let accounts = default_accounts();
    let initial_balance = 10_000.into();
    let mock_deposit_value = 1_000.into();
    let close_duration = 360_000;
    set_contract_balance(accounts.alice, initial_balance);
    set_contract_balance(accounts.bob, initial_balance);

    // when
    set_next_caller(accounts.alice);
    let contract_id = contract_id();
    let payment_channel = PaymentChannel::new(accounts.bob, close_duration);
    set_contract_balance(contract_id, mock_deposit_value);

    // then
    assert_eq!(payment_channel.get_sender(), accounts.alice);
    assert_eq!(payment_channel.get_recipient(), accounts.bob);
    assert_eq!(payment_channel.get_balance(), mock_deposit_value);
    assert_eq!(payment_channel.get_close_duration(), close_duration);
    assert_eq!(payment_channel.get_withdrawn(), U256::zero());
}