// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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
    db::ExecContext,
    AccountError,
    EmittedEvent,
    EnvInstance,
    OnInstance,
};
use crate::env3::{
    call::CallData,
    EnvTypes,
    Result,
};
use ink_prelude::string::String;

/// Pushes a contract execution context.
///
/// This is the data behind a single instance of a contract call.
///
/// # Note
///
/// Together with [`pop_execution_context`] this can be used to emulated
/// nested calls.
pub fn push_execution_context<T>(
    caller: T::AccountId,
    callee: T::AccountId,
    gas_limit: T::Balance,
    endowment: T::Balance,
    call_data: CallData,
) where
    T: EnvTypes,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.exec_context.push(
            ExecContext::build::<T>()
                .caller(caller)
                .callee(callee)
                .gas(gas_limit)
                .transferred_value(endowment)
                .call_data(call_data)
                .finish(),
        )
    })
}

/// Pops the top contract execution context.
///
/// # Note
///
/// Together with [`push_execution_context`] this can be used to emulated
/// nested calls.
pub fn pop_execution_context() {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.exec_context.pop();
    })
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
pub fn set_account_balance<T>(
    account_id: T::AccountId,
    new_balance: T::Balance,
) -> Result<()>
where
    T: EnvTypes,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .accounts
            .get_account_mut::<T>(&account_id)
            .ok_or_else(|| AccountError::no_account_for_id::<T>(&account_id))
            .map_err(Into::into)
            .and_then(|account| account.set_balance::<T>(new_balance).map_err(Into::into))
    })
}

/// Returns the balance of the account.
///
/// # Note
///
/// Note that account could refer to either a user account or
/// a smart contract account. This returns the same as `env::api::balance`
/// if given the account ID of the currently executed smart contract.
///
/// # Errors
///
/// - If `account` does not exist.
/// - If the underlying `account` type does not match.
pub fn get_account_balance<T>(account_id: T::AccountId) -> Result<T::Balance>
where
    T: EnvTypes,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .accounts
            .get_account::<T>(&account_id)
            .ok_or_else(|| AccountError::no_account_for_id::<T>(&account_id))
            .map_err(Into::into)
            .and_then(|account| account.balance::<T>().map_err(Into::into))
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
    account_id: T::AccountId,
    new_rent_allowance: T::Balance,
) -> Result<()>
where
    T: EnvTypes,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .accounts
            .get_account_mut::<T>(&account_id)
            .ok_or_else(|| AccountError::no_account_for_id::<T>(&account_id))
            .map_err(Into::into)
            .and_then(|account| {
                account
                    .set_rent_allowance::<T>(new_rent_allowance)
                    .map_err(Into::into)
            })
    })
}

/// Returns the rent allowance of the contract account.
///
/// # Errors
///
/// - If `account` does not exist.
/// - If the underlying `account` type does not match.
/// - If the returned rent allowance cannot be properly decoded.
pub fn get_contract_rent_allowance<T>(account_id: T::AccountId) -> Result<T::Balance>
where
    T: EnvTypes,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .accounts
            .get_account::<T>(&account_id)
            .ok_or_else(|| AccountError::no_account_for_id::<T>(&account_id))
            .map_err(Into::into)
            .and_then(|account| account.rent_allowance::<T>().map_err(Into::into))
    })
}

/// Creates a new user account and returns its account ID.
///
/// # Errors
///
/// - If `initial_balance` cannot be properly encoded.
pub fn create_user_account<T>(initial_balance: T::Balance) -> Result<T::AccountId>
where
    T: EnvTypes,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        Ok(instance.accounts.new_user_account::<T>(initial_balance))
    })
}

/// Sets the runtime storage to value for the given key.
pub fn set_runtime_storage<T>(key: &[u8], value: T)
where
    T: scale::Encode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.runtime_storage.store(key.to_vec(), value)
    })
}

/// Sets the call handler for runtime calls.
pub fn set_runtime_call_handler<T, F>(f: F)
where
    T: EnvTypes,
    F: FnMut(<T as EnvTypes>::Call) + 'static,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.runtime_call_handler.register::<T, F>(f)
    })
}

/// Set the entropy hash of the current block.
///
/// # Note
///
/// This allows to control what [`crate::env3::random`] returns.
pub fn set_block_entropy<T>(entropy: T::Hash) -> Result<()>
where
    T: EnvTypes,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.current_block_mut()?.set_entropy::<T>(entropy)
    })
    .map_err(Into::into)
}

/// Returns the contents of the past performed environmental `println` in order.
pub fn recorded_printlns() -> impl Iterator<Item = String> {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        // We return a clone of the recorded strings instead of
        // references to them since this would require the whole `on_instance`
        // API to operate on `'static` environmental instances which would
        // ultimately allow leaking those `'static` references to the outside
        // and potentially lead to terrible bugs such as iterator invalidation.
        instance
            .console
            .past_prints()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
            .into_iter()
    })
}

/// Returns the recorded emitted events in order.
pub fn recorded_events() -> impl Iterator<Item = EmittedEvent> {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        // We return a clone of the recorded emitted events instead of
        // references to them since this would require the whole `on_instance`
        // API to operate on `'static` environmental instances which would
        // ultimately allow leaking those `'static` references to the outside
        // and potentially lead to terrible bugs such as iterator invalidation.
        instance
            .emitted_events
            .emitted_events()
            .map(Clone::clone)
            .collect::<Vec<_>>()
            .into_iter()
    })
}

/// Advances the chain by a single block.
pub fn advance_block() {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.advance_block()
    })
}
