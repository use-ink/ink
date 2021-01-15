// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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
use crate::engine::off_chain::OffChainError;

/// Errors that can be encountered upon environmental interaction.
#[derive(Debug, From, PartialEq, Eq)]
pub enum Error {
    /// Error upon decoding an encoded value.
    Decode(scale::Error),
    /// An error that can only occure in the off-chain environment.
    #[cfg(any(feature = "std", test, doc))]
    OffChain(OffChainError),
    /// The call to another contract has trapped.
    CalleeTrapped,
    /// The call to another contract has been reverted.
    CalleeReverted,
    /// The queried contract storage entry is missing.
    KeyNotFound,
    /// Transfer failed because it would have brought the sender's total balance
    /// below the subsistence threshold.
    BelowSubsistenceThreshold,
    /// Transfer failed for other not further specified reason. Most probably
    /// reserved or locked balance of the sender that was preventing the transfer.
    TransferFailed,
    /// The newly created contract is below the subsistence threshold after executing
    /// its constructor so no usable contract instance will be created.
    NewContractNotFunded,
    /// No code could be found at the supplied code hash.
    CodeNotFound,
    /// The account that was called is either no contract (e.g. user account) or is a tombstone.
    NotCallable,
    /// An unknown error has occured.
    UnknownError,
}

/// A result of environmental operations.
pub type Result<T> = core::result::Result<T, Error>;
