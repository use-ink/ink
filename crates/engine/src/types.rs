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

/// This is just a temporary solution for the MVP!
/// As a temporary solution we choose the same type as the default
/// `env` `Balance` type.
///
/// In the long-term this type should be `Vec<u8>` as well, as to not
/// be dependent on the specific off-chain environment type, so that
/// the `engine` crate can be used with an arbitrary `Environment` configuration.
pub type Balance = u128;

/// The Account Id type used by this crate.
#[derive(Debug, From, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct AccountId(Vec<u8>);

impl From<AccountId> for Vec<u8> {
    fn from(account_id: AccountId) -> Self {
        account_id.0
    }
}

impl From<&[u8]> for AccountId {
    fn from(slice: &[u8]) -> Self {
        AccountId(slice.to_vec())
    }
}

impl Default for AccountId {
    fn default() -> Self {
        Self(vec![0x01; 32])
    }
}

/// Key into the contract storage.
///
/// Used to identify contract storage cells for read and write operations.
#[derive(Default, From, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Key(Vec<u8>);

impl From<&Vec<u8>> for Key {
    fn from(vec: &Vec<u8>) -> Self {
        vec.clone().into()
    }
}

impl From<&[u8]> for Key {
    fn from(slice: &[u8]) -> Self {
        slice.to_vec().into()
    }
}

impl From<[u8; 32]> for Key {
    fn from(arr: [u8; 32]) -> Self {
        arr.to_vec().into()
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
