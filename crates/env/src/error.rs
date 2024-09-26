// Copyright (C) Use Ink (UK) Ltd.
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
use pallet_revive_uapi::ReturnErrorCode;

/// Errors that can be encountered upon environmental interaction.
#[derive(Debug, From, PartialEq, Eq)]
pub enum Error {
    /// Error upon decoding an encoded value.
    Decode(scale::Error),
    /// The static buffer used during ABI encoding or ABI decoding is too small.
    BufferTooSmall,
    /// An error that can only occur in the off-chain environment.
    #[cfg(any(feature = "std", test, doc))]
    OffChain(OffChainError),
    /// The error returned by the contract.
    ReturnError(ReturnErrorCode),
}

/// A result of environmental operations.
pub type Result<T> = core::result::Result<T, Error>;
