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

#![cfg_attr(not(feature = "std"), no_std)]

mod types;

pub use types::*;
pub mod arithmetic;

#[cfg(not(any(feature = "std", test, doc)))]
use derive_more::From;

#[cfg(feature = "std")]
pub type Result<T, X> = core::result::Result<T, Error<X>>;

#[cfg(not(feature = "std"))]
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can be encountered upon environmental interaction.
#[cfg(not(any(feature = "std", test, doc)))]
#[derive(Debug, From, PartialEq, Eq)]
pub enum Error {
    /// Error upon decoding an encoded value.
    Decode(scale::Error),
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

/// Errors that can be encountered upon environmental interaction.
#[cfg(any(feature = "std", test, doc))]
#[derive(Debug, PartialEq, Eq)]
pub enum Error<X> {
    /// Error upon decoding an encoded value.
    Decode(scale::Error),
    /// An error that can only occur in the off-chain environment.
    OffChain(X),
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

#[cfg(any(feature = "std", test, doc))]
impl<X> From<scale::Error> for Error<X> {
    fn from(err: scale::Error) -> Self {
        Error::Decode(err)
    }
}
