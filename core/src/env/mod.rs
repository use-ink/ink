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

mod api;
mod srml;
mod traits;

#[cfg(feature = "test-env")]
pub mod test;

#[cfg(feature = "test-env")]
mod test_env;

#[cfg(feature = "test-env")]
pub use test_env::TestEnv;

pub use api::*;
pub use traits::*;

pub use self::srml::DefaultSrmlTypes;

// TODO: [AJ] update doc comments

/// The environment implementation that is currently being used.
///
/// This may be either
/// - `DefaultEnv` for real contract storage
///   manipulation that may happen on-chain.
/// - `TestEnv` for emulating a contract environment
///   that can be inspected by the user and used
///   for testing contracts off-chain.
#[cfg(not(feature = "test-env"))]
pub type ContractEnvStorage = self::srml::SrmlEnvStorage;

/// The environment implementation that is currently being used.
#[cfg(feature = "test-env")]
pub(self) type ContractEnvStorage = self::test_env::TestEnvStorage;

#[cfg(not(feature = "test-env"))]
pub type ContractEnv<T> = self::srml::SrmlEnv<T>;

#[cfg(feature = "test-env")]
pub type ContractEnv<T> = self::test_env::TestEnv<T>;
