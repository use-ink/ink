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

use scale::{
    Decode,
    Encode,
};

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
