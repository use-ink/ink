// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use super::ContractEnvStorage;
use crate::{
    env::{
        traits::{
            Env,
            EnvTypes,
        },
        CallError,
        CreateError,
        EnvStorage as _,
    },
    memory::vec::Vec,
    storage::Key,
};
use scale::{
    Decode,
    Encode,
};

/// Stores the given value under the specified key in the contract storage.
///
/// # Safety
///
/// This operation is unsafe because it does not check for key integrity.
/// Users can compare this operation with a raw pointer dereferencing in Rust.
pub unsafe fn store(key: Key, value: &[u8]) {
    ContractEnvStorage::store(key, value)
}

/// Clears the data stored at the given key from the contract storage.
///
/// # Safety
///
/// This operation is unsafe because it does not check for key integrity.
/// Users can compare this operation with a raw pointer dereferencing in Rust.
pub unsafe fn clear(key: Key) {
    ContractEnvStorage::clear(key)
}

/// Loads the data stored at the given key from the contract storage.
///
/// # Safety
///
/// This operation is unsafe because it does not check for key integrity.
/// Users can compare this operation with a raw pointer dereferencing in Rust.
pub unsafe fn load(key: Key) -> Option<Vec<u8>> {
    ContractEnvStorage::load(key)
}

/// Returns the given data back to the caller.
///
/// # Note
///
/// This operation must be the last operation performed by a called
/// smart contract before it returns the execution back to its caller.
pub fn return_data<T, E>(data: T)
where
    T: Encode,
    E: Env,
{
    E::return_data(&data.encode()[..])
}

/// Dispatches a Call into the runtime, for invoking other substrate
/// modules. Dispatched only after successful contract execution.
///
/// The encoded Call MUST be decodable by the target substrate runtime.
/// If decoding fails, then the smart contract execution will fail.
pub fn dispatch_call<T, C>(call: C)
where
    T: Env,
    C: Into<<T as EnvTypes>::Call>,
{
    T::dispatch_raw_call(&call.into().encode()[..])
}

/// Invokes a remote smart contract.
///
/// Does not expect to receive return data back.
/// Use this whenever you call a remote smart contract that returns nothing back.
pub fn call_invoke<T>(
    callee: T::AccountId,
    gas: u64,
    value: T::Balance,
    input_data: &[u8],
) -> Result<(), CallError>
where
    T: Env,
{
    T::call_invoke(callee, gas, value, input_data)
}

/// Evaluates a remote smart contract.
///
/// Expects to receive return data back.
/// Use this whenever calling a remote smart contract that returns a value.
pub fn call_evaluate<T, R>(
    callee: T::AccountId,
    gas: u64,
    value: T::Balance,
    input_data: &[u8],
) -> Result<R, CallError>
where
    T: Env,
    R: Decode,
{
    T::call_evaluate(callee, gas, value, input_data)
}

/// Instantiates a new smart contract.
///
/// Upon success returns the account ID of the newly created smart contract.
pub fn create<T>(
    code_hash: T::Hash,
    gas_limit: u64,
    value: T::Balance,
    input_data: &[u8],
) -> Result<T::AccountId, CreateError>
where
    T: Env,
{
    T::create(code_hash, gas_limit, value, input_data)
}
