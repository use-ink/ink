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

use crate::env3::{
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
) -> Result<()>
where
    T: EnvTypes,
{
    todo!()
}

/// Pops the top contract execution context.
///
/// # Note
///
/// Together with [`push_execution_context`] this can be used to emulated
/// nested calls.
pub fn pop_execution_context() {
    todo!()
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
pub fn set_balance<T>(account: T::AccountId, new_balance: T::Balance) -> Result<()>
where
    T: EnvTypes,
{
    todo!()
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
pub fn get_balance<T>(account: T::AccountId) -> T::Balance
where
    T: EnvTypes,
{
    todo!()
}

/// Sets the rent allowance of the contract account to the given rent allowance.
///
/// # Errors
///
/// - If `account` does not exist.
/// - If the underlying `account` type does not match.
/// - If the underlying `new_rent_allowance` type does not match.
pub fn set_rent_allowance<T>(
    account: T::AccountId,
    new_rent_allowance: T::Balance,
) -> Result<()>
where
    T: EnvTypes,
{
    todo!()
}

/// Returns the rent allowance of the contract account.
///
/// # Errors
///
/// - If `account` does not exist.
/// - If the underlying `account` type does not match.
/// - If the returned rent allowance cannot be properly decoded.
pub fn get_rent_allowance<T>(account: T::AccountId) -> Result<T::Balance>
where
    T: EnvTypes,
{
    todo!()
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
