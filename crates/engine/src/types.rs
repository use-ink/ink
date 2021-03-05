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

/// Off-chain environment account ID type.
#[derive(Debug, From, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

/// Key into contract storage.
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
