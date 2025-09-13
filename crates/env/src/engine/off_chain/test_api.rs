// Copyright (C) Use Ink (UK) Ltd.
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
    Result,
    types::Environment,
};
use core::fmt::Debug;
use std::panic::UnwindSafe;

pub use super::call_data::CallData;
pub use ink_engine::ext::ChainSpec;
use ink_primitives::{
    AccountIdMapper,
    Address,
    H256,
    U256,
};

/// Record for an emitted event.
#[derive(Clone)]
pub struct EmittedEvent {
    /// Recorded topics of the emitted event.
    pub topics: Vec<[u8; 32]>,
    /// Recorded encoding of the emitted event.
    pub data: Vec<u8>,
}

/// Sets the balance of a contract to the given balance.
///
/// # Note
///
/// If a 0 balance is set, this would not fail. This is useful for
/// reaping an account.
///
/// # Errors
///
/// - If `addr` does not exist.
/// - If the underlying `new_balance` type does not match.
/// - If the `new_balance` is less than the existential minimum.
pub fn set_contract_balance(addr: Address, new_balance: U256) {
    let min = ChainSpec::default().minimum_balance;
    if new_balance < min && new_balance != U256::zero() {
        panic!("Balance must be at least [{min}]. Use 0 as balance to reap the account.");
    }

    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.set_balance(addr, new_balance);
    })
}

/// Returns the balance of a contract.
///
/// # Note
///
/// This returns the same as `env::api::balance` if given the contract
/// address of the currently executed smart contract.
///
/// # Errors
///
/// - If `contract` does not exist.
pub fn get_contract_balance<T>(addr: Address) -> Result<U256> {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.get_balance(addr).map_err(Into::into)
    })
}

/// Set to true to disable clearing storage
///
/// # Note
///
/// Useful for benchmarks because it ensures the initialized storage is maintained across
/// runs, because lazy storage structures automatically clear their associated cells when
/// they are dropped.
pub fn set_clear_storage_disabled(_disable: bool) {
    unimplemented!(
        "off-chain environment does not yet support `set_clear_storage_disabled`"
    );
}

/// Advances the chain by a single block.
pub fn advance_block<T>()
where
    T: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.advance_block();
    })
}

/// Sets a caller for the next call.
pub fn set_caller(caller: Address) {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.set_caller(caller);
    })
}

/// Sets the callee for the next call.
pub fn set_callee(callee: Address) {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.set_callee(callee);
    })
}

/// Sets an account as a contract
pub fn set_contract(contract: Address) {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.set_contract(contract);
    })
}

/// Returns a boolean to indicate whether an account is a contract
#[cfg(feature = "unstable-hostfn")]
pub fn is_contract(contract: Address) -> bool {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.is_contract(&contract)
    })
}

/// Gets the currently set callee.
///
/// This is the address of the currently executing contract.
pub fn callee() -> Address {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        let callee = instance.engine.get_callee();
        scale::Decode::decode(&mut &callee[..])
            .unwrap_or_else(|err| panic!("encoding failed: {err}"))
    })
}

/// Returns the total number of reads and writes of the contract's storage.
pub fn get_contract_storage_rw(addr: Address) -> (usize, usize) {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.get_contract_storage_rw(addr)
    })
}

/// Sets the value transferred from the caller to the callee as part of the call.
///
/// Please note that the acting accounts should be set with [`set_caller()`] and
/// [`set_callee()`] beforehand.
pub fn set_value_transferred(value: U256) {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.set_value_transferred(value);
    })
}

/// Transfers value from the caller account to the contract.
///
/// Please note that the acting accounts should be set with [`set_caller()`] and
/// [`set_callee()`] beforehand.
#[allow(clippy::arithmetic_side_effects)] // todo
pub fn transfer_in(value: U256) {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        let caller = instance.engine.exec_context.caller;

        let caller_old_balance = instance.engine.get_balance(caller).unwrap_or_default();

        let callee = instance.engine.get_callee();
        let contract_old_balance =
            instance.engine.get_balance(callee).unwrap_or_default();

        instance
            .engine
            .set_balance(caller, caller_old_balance - value);
        instance
            .engine
            .set_balance(callee, contract_old_balance + value);
        instance.engine.set_value_transferred(value);
    });
}

/// Returns the amount of storage cells used by the contract `addr`.
///
/// Returns `None` if the contract at `addr` is non-existent.
pub fn count_used_storage_cells<T>(addr: Address) -> Result<usize>
where
    T: Environment,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .engine
            .count_used_storage_cells(&addr)
            .map_err(Into::into)
    })
}

/// Sets the block timestamp for the next [`advance_block`] invocation.
pub fn set_block_timestamp<T>(value: T::Timestamp)
where
    T: Environment<Timestamp = u64>,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.set_block_timestamp(value);
    })
}

/// Sets the block number for the next [`advance_block`] invocation.
pub fn set_block_number<T>(value: T::BlockNumber)
where
    T: Environment<BlockNumber = u32>,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.set_block_number(value);
    })
}

/// Runs the given closure test function with the default configuration
/// for the off-chain environment.
pub fn run_test<T, F>(f: F) -> Result<()>
where
    T: Environment,
    F: FnOnce(DefaultAccounts) -> Result<()>,
{
    let default_accounts = default_accounts();
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.engine.initialize_or_reset();

        let alice = default_accounts.alice;
        // instance.engine.set_caller(alice.clone()); // todo
        instance.engine.set_callee(alice);

        // set up the funds for the default accounts
        let substantial = 1_000_000.into();
        let some = 1_000.into();
        instance.engine.set_balance(alice, substantial);
        instance.engine.set_balance(default_accounts.bob, some);
        instance.engine.set_balance(default_accounts.charlie, some);
        instance
            .engine
            .set_balance(default_accounts.django, 0.into());
        instance.engine.set_balance(default_accounts.eve, 0.into());
        instance
            .engine
            .set_balance(default_accounts.frank, 0.into());
    });
    f(default_accounts)
}

/// Returns the `H160` addresses of default accounts, for testing
/// purposes: Alice, Bob, Charlie, Django, Eve and Frank.
pub fn default_accounts() -> DefaultAccounts {
    DefaultAccounts {
        alice: AccountIdMapper::to_address(&[0x01; 32]),
        bob: AccountIdMapper::to_address(&[0x02; 32]),
        charlie: AccountIdMapper::to_address(&[0x03; 32]),
        django: AccountIdMapper::to_address(&[0x04; 32]),
        eve: AccountIdMapper::to_address(&[0x05; 32]),
        frank: AccountIdMapper::to_address(&[0x06; 32]),
    }
}

/// Addresses of the default accounts.
pub struct DefaultAccounts {
    /// The predefined `ALICE` address holding substantial amounts of value.
    pub alice: Address,
    /// The predefined `BOB` address holding some amounts of value.
    pub bob: Address,
    /// The predefined `CHARLIE` address holding some amounts of value.
    pub charlie: Address,
    /// The predefined `DJANGO` address holding no value.
    pub django: Address,
    /// The predefined `EVE` address holding no value.
    pub eve: Address,
    /// The predefined `FRANK` address holding no value.
    pub frank: Address,
}

/// Returns the recorded emitted events in order.
pub fn recorded_events() -> Vec<EmittedEvent> {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .engine
            .get_emitted_events()
            .map(|evt: ink_engine::test_api::EmittedEvent| evt.into())
            .collect()
    })
}

/// Tests if a contract terminates successfully after `self.env().terminate()`
/// has been called.
///
/// The arguments denote:
///
/// * `should_terminate`: A closure in which the function supposed to terminate is called.
/// * `expected_beneficiary`: The beneficiary account who should have received the
///   remaining value in the contract
/// * `expected_value_transferred_to_beneficiary`: The value which should have been
///   transferred to the `expected_beneficiary`.
///
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
/// See our [`contract-terminate`](https://github.com/use-ink/ink-examples/tree/v5.x.x/contract-terminate)
/// example for a complete usage exemplification.
pub fn assert_contract_termination<T, F>(
    should_terminate: F,
    expected_beneficiary: Address,
    expected_value_transferred_to_beneficiary: U256,
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
    let (value_transferred, beneficiary): (U256, Address) =
        scale::Decode::decode(&mut &encoded_input[..])
            .unwrap_or_else(|err| panic!("input can not be decoded: {err}"));
    assert_eq!(value_transferred, expected_value_transferred_to_beneficiary);
    assert_eq!(beneficiary, expected_beneficiary);
}

/// Prepend contract message call with value transfer. Used for tests in off-chain
/// environment.
#[macro_export]
macro_rules! pay_with_call {
    ($contract:ident . $message:ident ( $( $params:expr ),* ) , $amount:expr) => {{
        $crate::test::transfer_in($amount);
        $contract.$message($ ($params) ,*)
    }}
}

/// Retrieves the value stored by `return_value()`.
pub fn get_return_value() -> Vec<u8> {
    <EnvInstance as OnInstance>::on_instance(|instance| instance.get_return_value())
}

/// Gets a pseudo code hash for a contract ref.
pub fn upload_code<E, ContractRef>() -> H256
where
    E: Environment,
    ContractRef: crate::ContractReverseReference,
    <ContractRef as crate::ContractReverseReference>::Type:
        crate::reflect::ContractMessageDecoder,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.upload_code::<ContractRef>()
    })
}
