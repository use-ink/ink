// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    memory::vec::Vec,
    storage::Key,
};
use parity_codec::Codec;

/// The environmental types usable by contracts defined with pDSL.
pub trait EnvTypes {
    /// The type of an address.
    type Address: Codec + PartialEq + Eq;
    /// The type of balances.
    type Balance: Codec;
    /// The type of hash.
    type Hash: Codec;
}

/// Types implementing this can act as contract storage.
pub trait EnvStorage {
    /// Stores the given value under the given key.
    ///
    /// # Safety
    ///
    /// This operation is unsafe becaues it does not check for key integrity.
    /// Users can compare this operation with a raw pointer dereferencing in Rust.
    unsafe fn store(key: Key, value: &[u8]);

    /// Clears the value stored under the given key.
    ///
    /// # Safety
    ///
    /// This operation is unsafe becaues it does not check for key integrity.
    /// Users can compare this operation with a raw pointer dereferencing in Rust.
    unsafe fn clear(key: Key);

    /// Loads data stored under the given key.
    ///
    /// # Safety
    ///
    /// This operation is unsafe becaues it does not check for key integrity.
    /// Users can compare this operation with a raw pointer dereferencing in Rust.
    unsafe fn load(key: Key) -> Option<Vec<u8>>;
}

/// The evironment API usable by contracts defined with pDSL.
pub trait Env: EnvTypes + EnvStorage {
    /// Returns the chain address of the caller.
    fn caller() -> <Self as EnvTypes>::Address;

    /// Loads input data for contract execution.
    fn input() -> Vec<u8>;

    /// Get the latest block RNG seed
    fn random_seed() -> <Self as EnvTypes>::Hash;

    /// Returns from the contract execution with the given value.
    ///
    /// # Safety
    ///
    /// The external callers rely on the correct type of the encoded
    /// returned value. This API is unsafe because it does not provide
    /// guarantees on its own to always encode the expected type.
    unsafe fn r#return(value: &[u8]) -> !;

    /// Prints the given content to Substrate output.
    ///
    /// # Note
    ///
    /// Usable only in development (`--dev`) chains.
    fn println(content: &str);
}
