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
mod calls;
mod srml;
mod traits;

#[cfg(feature = "test-env")]
pub mod test;

#[cfg(feature = "test-env")]
mod test_env;

pub use api::*;
pub use traits::*;

pub use self::{
    calls::{
        CallBuilder,
        CreateBuilder,
        FromAccountId,
        ReturnType,
    },
    srml::DefaultSrmlTypes,
    traits::{
        CallError,
        CreateError,
    },
};

/// The storage environment implementation that is currently being used.
///
/// This may be either
/// - `SrmlEnvStorage` for real contract storage
///   manipulation that may happen on-chain.
/// - `TestEnvStorage` for emulating a contract environment
///   that can be inspected by the user and used
///   for testing contracts off-chain.
#[cfg(not(feature = "test-env"))]
pub(self) type ContractEnvStorage = self::srml::SrmlEnvStorage;

/// The storage environment implementation for the test environment.
#[cfg(feature = "test-env")]
pub(self) type ContractEnvStorage = self::test_env::TestEnvStorage;

/// The contract environment implementation that is currently being used
///
/// Generic over user supplied EnvTypes for different runtimes
#[cfg(not(feature = "test-env"))]
pub type ContractEnv<T> = self::srml::SrmlEnv<T>;

/// The contract environment implementation for the test environment
#[cfg(feature = "test-env")]
pub type ContractEnv<T> = self::test_env::TestEnv<T>;
