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

use derive_more::From;

#[cfg(any(feature = "std", test, doc))]
use crate::env::engine::off_chain::OffChainError;

/// Errors that can be encountered upon environmental interaction.
#[derive(Debug, From, PartialEq, Eq)]
pub enum EnvError {
    /// Error upon decoding an encoded value.
    Decode(scale::Error),
    /// An error that can only occure in the off-chain environment.
    #[cfg(any(feature = "std", test, doc))]
    OffChain(OffChainError),
    /// The call to another contract has trapped.
    ContractCallTrapped,
    /// A called contract returned a custom error code.
    #[from(ignore)]
    ContractCallFailState(u8),
    /// The instantiation of another contract has trapped.
    ContractInstantiationTrapped,
    /// The instantiated contract returned a custom error code.
    #[from(ignore)]
    ContractInstantiationFailState(u8),
    /// The queried runtime storage entry is missing.
    MissingRuntimeStorageEntry,
    /// The queried contract storage entry is missing.
    MissingContractStorageEntry,
    /// A call to transfer value from the contract failed.
    TransferCallFailed,
}

/// A result of environmental operations.
pub type Result<T> = core::result::Result<T, EnvError>;
