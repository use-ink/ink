// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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
    EnvInstance,
};
use crate::env3::{
    call::CallData,
    engine::OnInstance,
    EnvError,
    EnvTypes,
    Result,
};

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
pub fn set_balance<T>(account_id: T::AccountId, new_balance: T::Balance) -> Result<()>
where
    T: EnvTypes,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .accounts
            .get_account_mut::<T>(account_id)
            .ok_or(EnvError::OffChain)
            .and_then(|account| {
                account.set_balance::<T>(new_balance).map_err(|_| EnvError::OffChain)
            })
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
pub fn get_balance<T>(account_id: T::AccountId) -> Result<T::Balance>
where
    T: EnvTypes,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .accounts
            .get_account::<T>(account_id)
            .ok_or(EnvError::OffChain)
            .and_then(|account| {
                account.balance::<T>().map_err(|_| EnvError::OffChain)
            })
    })
}

/// Sets the rent allowance of the contract account to the given rent allowance.
///
/// # Errors
///
/// - If `account` does not exist.
/// - If the underlying `account` type does not match.
/// - If the underlying `new_rent_allowance` type does not match.
pub fn set_rent_allowance<T>(
    account_id: T::AccountId,
    new_rent_allowance: T::Balance,
) -> Result<()>
where
    T: EnvTypes,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .accounts
            .get_account_mut::<T>(account_id)
            .ok_or(EnvError::OffChain)
            .and_then(|account| {
                account.set_rent_allowance::<T>(new_rent_allowance).map_err(|_| EnvError::OffChain)
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
pub fn get_rent_allowance<T>(account_id: T::AccountId) -> Result<T::Balance>
where
    T: EnvTypes,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance
            .accounts
            .get_account::<T>(account_id)
            .ok_or(EnvError::OffChain)
            .and_then(|account| {
                account.rent_allowance::<T>().map_err(|_| EnvError::OffChain)
            })
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
    todo!()
}
