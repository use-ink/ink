// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

//! Right now the `engine` crate can only be used with the `ink_env::DefaultEnvironment`.
//!
//! In the long-term all types in this module should be `Vec<u8>`, as to not
//! be dependent on any specific off-chain environment type. Then the `engine`
//! crate could be used with an arbitrary `Environment` configuration.
//!
//! Right now the issue is that if e.g. some test calls an `engine` API function
//! to transfer some balance, then there has to be some `+` operation to the
//! existing balance. And we can't just add a `Vec<u8>` onto another one, we
//! need to know the type.

use derive_more::From;

/// Same type as the `DefaultEnvironment::Hash` type.
pub type Hash = [u8; 32];

/// As a temporary solution we choose the same type as the `DefaultEnvironment::BlockNumber` type.
pub type BlockNumber = u32;

/// As a temporary solution we choose the same type as the `DefaultEnvironment::BlockTimestamp` type.
pub type BlockTimestamp = u64;

/// As a temporary solution we choose the same type as the `DefaultEnvironment::Balance` type.
pub type Balance = u128;

/// The Account Id type used by this crate.
#[derive(Debug, From, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct AccountId(Vec<u8>);

impl AccountId {
    /// Creates a new `AccountId` from the given raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }

    /// Returns the `AccountId` as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

/// Key into the database.
///
/// Used to identify contract storage cells for read and write operations.
#[derive(Default, From, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Key(Vec<u8>);

impl Key {
    /// Creates a new `Key` from the given raw bytes.
    #[allow(dead_code)]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

/// Errors encountered upon interacting with accounts.
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub enum AccountError {
    Decoding(scale::Error),
    #[from(ignore)]
    UnexpectedUserAccount,
    #[from(ignore)]
    NoAccountForId(Vec<u8>),
}
