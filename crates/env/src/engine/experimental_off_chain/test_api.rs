// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Operations on the off-chain testing environment.

use super::{
    EnvInstance,
    OnInstance,
};
use crate::{
    Environment,
    Result,
};
use core::fmt::Debug;
use ink_engine::test_api::RecordedDebugMessages;
use std::panic::UnwindSafe;

/// Record for an emitted event.
#[derive(Clone)]
pub struct EmittedEvent {
    /// Recorded topics of the emitted event.
    pub topics: Vec<Vec<u8>>,
    /// Recorded encoding of the emitted event.
    pub data: Vec<u8>,
}

/// Sets the balance of the account to the given balance.
///
/// # Note
///
/// Note that account could refer to either a user account or
/// a smart contract account.
///
/// # Errors
///
/// - If `account` does not exist.
/// - If the underlying `account` type does not match.
/// - If the underlying `new_balance` type does not match.
pub fn set_account_balance<T>(account_id: T::AccountId, new_balance: T::Balance)
where
    T: Environment<Balance = u128>, // Just temporary for the MVP!
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .engine
            .set_balance(scale::Encode::encode(&account_id), new_balance);
    })
}

/// Returns the balance of the account.
///
/// # Note
///
/// Note that account could refer to either a user account or
/// a smart contract account. This returns the same as `env::api::balance`
/// if given the account id of the currently executed smart contract.
///
/// # Errors
///
/// - If `account` does not exist.
/// - If the underlying `account` type does not match.
pub fn get_account_balance<T>(account_id: T::AccountId) -> Result<T::Balance>
where
    T: Environment<Balance = u128>, // Just temporary for the MVP!
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .engine
            .get_balance(scale::Encode::encode(&account_id))
            .map_err(Into::into)
    })
}

/// Sets the rent allowance of the contract account to the given rent allowance.
///
/// # Errors
///
/// - If `account` does not exist.
/// - If the underlying `account` type does not match.
/// - If the underlying `new_rent_allowance` type does not match.
pub fn set_contract_rent_allowance<T>(
    _account_id: T::AccountId,
    _new_rent_allowance: T::Balance,
) -> Result<()>
where
    T: Environment,
{
    unimplemented!(
        "off-chain environment does not yet support `set_contract_rent_allowance`"
    );
}

/// Returns the rent allowance of the contract account.
///
/// # Errors
///
/// - If `account` does not exist.
/// - If the underlying `account` type does not match.
/// - If the returned rent allowance cannot be properly decoded.
pub fn get_contract_rent_allowance<T>(_account_id: T::AccountId) -> Result<T::Balance>
where
    T: Environment,
{
    unimplemented!(
        "off-chain environment does not yet support `get_contract_rent_allowance`"
    );
}

/// Set the entropy hash of the current block.
///
/// # Note
///
/// This allows to control what [`random`][`crate::random`] returns.
pub fn set_block_entropy<T>(_entropy: T::Hash) -> Result<()>
where
    T: Environment,
{
    unimplemented!("off-chain environment does not yet support `set_block_entropy`");
}

/// Returns the contents of the past performed environmental debug messages in order.
pub fn recorded_debug_messages() -> RecordedDebugMessages {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.get_emitted_debug_messages()
    })
}

/// Set to true to disable clearing storage
///
/// # Note
///
/// Useful for benchmarks because it ensures the initialized storage is maintained across runs,
/// because lazy storage structures automatically clear their associated cells when they are dropped.
pub fn set_clear_storage_disabled(_disable: bool) {
    unimplemented!(
        "off-chain environment does not yet support `set_clear_storage_disabled`"
    );
}

/// Sets a caller for the next call.
pub fn set_caller<T>(caller: T::AccountId)
where
    T: Environment,
    <T as Environment>::AccountId: From<[u8; 32]>,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.set_caller(scale::Encode::encode(&caller));
    })
}

/// Sets the callee for the next call.
pub fn set_callee<T>(callee: T::AccountId)
where
    T: Environment,
    <T as Environment>::AccountId: From<[u8; 32]>,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.set_callee(scale::Encode::encode(&callee));
    })
}

/// Gets the currently set callee.
///
/// This is account id of the currently executing contract.
pub fn callee<T>() -> T::AccountId
where
    T: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        let callee = instance.engine.get_callee();
        scale::Decode::decode(&mut &callee[..]).expect("encoding failed")
    })
}

/// Returns the total number of reads and writes of the contract's storage.
pub fn get_contract_storage_rw<T>(account_id: &T::AccountId) -> (usize, usize)
where
    T: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .engine
            .get_contract_storage_rw(scale::Encode::encode(&account_id))
    })
}

/// Sets the balance of `account_id` to `new_balance`.
pub fn set_balance<T>(account_id: T::AccountId, new_balance: T::Balance)
where
    T: Environment<Balance = u128>, // Just temporary for the MVP!
    <T as Environment>::AccountId: From<[u8; 32]>,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .engine
            .set_balance(scale::Encode::encode(&account_id), new_balance);
    })
}

/// Sets the value transferred from the caller to the callee as part of the call.
pub fn set_value_transferred<T>(value: T::Balance)
where
    T: Environment<Balance = u128>, // Just temporary for the MVP!
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.set_value_transferred(value);
    })
}

/// Returns the amount of storage cells used by the account `account_id`.
///
/// Returns `None` if the `account_id` is non-existent.
pub fn count_used_storage_cells<T>(account_id: &T::AccountId) -> Result<usize>
where
    T: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .engine
            .count_used_storage_cells(&scale::Encode::encode(&account_id))
            .map_err(Into::into)
    })
}

/// Runs the given closure test function with the default configuration
/// for the off-chain environment.
pub fn run_test<T, F>(f: F) -> Result<()>
where
    T: Environment,
    F: FnOnce(DefaultAccounts<T>) -> Result<()>,
    <T as Environment>::AccountId: From<[u8; 32]>,
{
    let default_accounts = default_accounts::<T>();
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.initialize_or_reset();

        let encoded_alice = scale::Encode::encode(&default_accounts.alice);
        instance.engine.set_caller(encoded_alice.clone());
        instance.engine.set_callee(encoded_alice.clone());

        // set up the funds for the default accounts
        let substantial = 1_000_000;
        let some = 1_000;
        instance.engine.set_balance(encoded_alice, substantial);
        instance
            .engine
            .set_balance(scale::Encode::encode(&default_accounts.bob), some);
        instance
            .engine
            .set_balance(scale::Encode::encode(&default_accounts.charlie), some);
        instance
            .engine
            .set_balance(scale::Encode::encode(&default_accounts.django), 0);
        instance
            .engine
            .set_balance(scale::Encode::encode(&default_accounts.eve), 0);
        instance
            .engine
            .set_balance(scale::Encode::encode(&default_accounts.frank), 0);
    });
    f(default_accounts)
}

/// Returns the default accounts for testing purposes:
/// Alice, Bob, Charlie, Django, Eve and Frank.
pub fn default_accounts<T>() -> DefaultAccounts<T>
where
    T: Environment,
    <T as Environment>::AccountId: From<[u8; 32]>,
{
    DefaultAccounts {
        alice: T::AccountId::from([0x01; 32]),
        bob: T::AccountId::from([0x02; 32]),
        charlie: T::AccountId::from([0x03; 32]),
        django: T::AccountId::from([0x04; 32]),
        eve: T::AccountId::from([0x05; 32]),
        frank: T::AccountId::from([0x06; 32]),
    }
}

/// The default accounts.
pub struct DefaultAccounts<T>
where
    T: Environment,
{
    /// The predefined `ALICE` account holding substantial amounts of value.
    pub alice: T::AccountId,
    /// The predefined `BOB` account holding some amounts of value.
    pub bob: T::AccountId,
    /// The predefined `CHARLIE` account holding some amounts of value.
    pub charlie: T::AccountId,
    /// The predefined `DJANGO` account holding no value.
    pub django: T::AccountId,
    /// The predefined `EVE` account holding no value.
    pub eve: T::AccountId,
    /// The predefined `FRANK` account holding no value.
    pub frank: T::AccountId,
}

/// Returns the recorded emitted events in order.
pub fn recorded_events() -> impl Iterator<Item = EmittedEvent> {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .engine
            .get_emitted_events()
            .into_iter()
            .map(|evt: ink_engine::test_api::EmittedEvent| evt.into())
    })
}

/// Tests if a contract terminates successfully after `self.env().terminate()`
/// has been called.
///
/// The arguments denote:
///
/// * `should_terminate`: A closure in which the function supposed to terminate is called.
/// * `expected_beneficiary`: The beneficiary account who should have received the
///    remaining value in the contract
/// * `expected_value_transferred_to_beneficiary`: The value which should have been transferred
///   to the `expected_beneficiary`.
/// # Usage
///
/// ```no_compile
/// let should_terminate = move || your_contract.fn_which_should_terminate();
/// ink_env::test::assert_contract_termination::<ink_env::DefaultEnvironment, _>(
///     should_terminate,
///     expected_beneficiary,
///     expected_value_transferred_to_beneficiary
/// );
/// ```
///
/// See `examples/contract-terminate` for a complete usage example.
pub fn assert_contract_termination<T, F>(
    should_terminate: F,
    expected_beneficiary: T::AccountId,
    expected_value_transferred_to_beneficiary: T::Balance,
) where
    T: Environment,
    F: FnMut() + UnwindSafe,
    <T as Environment>::AccountId: Debug,
    <T as Environment>::Balance: Debug,
{
    let value_any = ::std::panic::catch_unwind(should_terminate)
        .expect_err("contract did not terminate");
    let encoded_input = value_any
        .downcast_ref::<Vec<u8>>()
        .expect("panic object can not be cast");
    let (value_transferred, encoded_beneficiary): (T::Balance, Vec<u8>) =
        scale::Decode::decode(&mut &encoded_input[..]).expect("input can not be decoded");
    let beneficiary =
        <T::AccountId as scale::Decode>::decode(&mut &encoded_beneficiary[..])
            .expect("input can not be decoded");
    assert_eq!(value_transferred, expected_value_transferred_to_beneficiary);
    assert_eq!(beneficiary, expected_beneficiary);
}
