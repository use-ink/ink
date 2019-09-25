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
