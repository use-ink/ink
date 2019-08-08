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
        EnvStorage as _,
    },
    memory::vec::Vec,
    storage::Key,
};
use scale::Encode;

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

/// Returns the current smart contract exection back to the caller
/// and return the given encoded value.
///
/// # Safety
///
/// External callers rely on the correct type of the encoded returned value.
/// This operation is unsafe because it does not provide guarantees on its
/// own to always encode the expected type.
pub unsafe fn r#return<T, E>(value: T) -> !
where
    T: Encode,
    E: Env,
{
    E::r#return(&value.encode()[..])
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
