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

//! Error definitions specific to environment accesses.

use derive_more::From;

/// Error encountered by calling a remote contract.
///
/// # Note
///
/// This is currently just a placeholder for potential future error codes.
#[derive(Debug, Copy, Clone)]
pub struct CallError;

/// Error encountered upon creating and instantiation a new smart contract.
///
/// # Note
///
/// This is currently just a placeholder for potential future error codes.
#[derive(Debug, Copy, Clone)]
pub struct CreateError;

/// Errors that can be encountered while accessing the contract's environment.
#[derive(Debug, Clone, From)]
pub enum Error {
    Call(CallError),
    Create(CreateError),
    Codec(scale::Error),
    InvalidStorageKey,
    InvalidStorageRead,
    InvalidContractCall,
    InvalidContractCallReturn,
    InvalidContractInstantiation,
    InvalidContractInstantiationReturn,
    InvalidRandomSeed,
}

/// The environmental error type.
pub type Result<T> = core::result::Result<T, Error>;
