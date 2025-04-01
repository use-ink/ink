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

//! Right now the `engine` crate can only be used with the `ink_env::DefaultEnvironment`.
//! This is a known limitation that we want to address in the future.

use derive_more::From;
use ink_primitives::{
    AccountId,
    Address,
};

/// Same type as the `DefaultEnvironment::BlockNumber` type.
pub type BlockNumber = u32;

/// Same type as the `DefaultEnvironment::BlockTimestamp` type.
pub type BlockTimestamp = u64;

/// Same type as the `DefaultEnvironment::Balance` type.
pub type Balance = u128;

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

// todo rename the whole thing
/// Errors encountered upon interacting with accounts.
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub enum AccountError {
    Decoding(scale::Error),
    #[from(ignore)]
    UnexpectedUserAccount,
    #[from(ignore)]
    NoAccountForId(AccountId),
    NoContractForId(Address),
}

/// The type of origins supported by `pallet-revive`.
#[derive(Debug, Eq, Default, Clone, scale::Encode, scale::Decode, PartialEq)]
//#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
pub enum Origin {
    #[default]
    Root,
    Signed(Vec<u8>),
}

// impl Origin {
// Returns the AccountId of a Signed Origin or an error if the origin is Root.
// pub fn account_id(&self) -> Result<AccountId, ()> {
// match self {
// Origin::Signed(id) => {
// let mut arr = [0u8; 32];
// arr.copy_from_slice(id.as_slice());
// Ok(AccountId::from(arr))
// },
// Origin::Root => Err(()),
// }
// }
// }
//
