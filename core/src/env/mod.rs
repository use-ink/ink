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

//! Contract environments.
//!
//! A contract is able to operate on different environments.
//!
//! Currently the SRML environment operating directly on the
//! substrate runtime module library (SRML) and the test
//! environment for testing and inspecting contracts are
//! provided.
//!
//! By default the SRML environment is used.
//! To enable the test environment the `test-env` crate feature
//! has to be enabled.

pub mod srml;

#[cfg(feature = "test-env")]
mod test_env;

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
    // /// The type of gas.
    // type Gas: Codec;
}

/// The evironment API usable by contracts defined with pDSL.
pub trait Env: EnvTypes {
    /// Returns the chain address of the caller.
    fn caller() -> <Self as EnvTypes>::Address;
    /// Stores the given value under the given key.
    ///
    /// # Safety
    ///
    /// Is unsafe since there is no check for key integrity.
    /// This operation can be compared to a pointer deref in Rust
    /// which itself is also considered unsafe.
    unsafe fn store(key: Key, value: &[u8]);
    /// Clears the value stored under the given key.
    ///
    /// # Safety
    ///
    /// Is unsafe since there is no check for key integrity.
    /// This operation can be compared to a pointer deref in Rust
    /// which itself is also considered unsafe.
    unsafe fn clear(key: Key);
    /// Loads data stored under the given key.
    ///
    /// # Safety
    ///
    /// Is unsafe since there is no check for key integrity.
    /// This operation can be compared to a pointer deref in Rust
    /// which itself is also considered unsafe.
    unsafe fn load(key: Key) -> Option<Vec<u8>>;
    /// Loads input data for contract execution.
    fn input() -> Vec<u8>;
    /// Returns from the contract execution with the given value.
    fn return_(value: &[u8]) -> !;
}

#[cfg(not(feature = "test-env"))]
pub use self::srml::SrmlEnv;

#[cfg(feature = "test-env")]
pub use self::test_env::TestEnv;

/// The environment implementation that is currently being used.
///
/// This may be either
/// - `DefaultEnv` for real contract storage
///   manipulation that may happen on-chain.
/// - `TestEnv` for emulating a contract environment
///   that can be inspected by the user and used
///   for testing contracts off-chain.
#[cfg(not(feature = "test-env"))]
pub type ContractEnv = self::srml::DefaultSrmlEnv;

/// The environment implementation that is currently being used.
#[cfg(feature = "test-env")]
pub type ContractEnv = self::test_env::TestEnv;
