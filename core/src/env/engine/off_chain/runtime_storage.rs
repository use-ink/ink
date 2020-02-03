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

use crate::env::Result;
use ink_prelude::{
    collections::btree_map::BTreeMap,
    vec::Vec,
};

/// Runtime storage.
///
/// More generically a mapping from bytes to bytes.
pub struct RuntimeStorage {
    /// The underlying storage mapping.
    entries: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl RuntimeStorage {
    /// Creates a new runtime storage.
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Stores the value under the given key.
    pub fn store<T>(&mut self, key: Vec<u8>, value: T)
    where
        T: scale::Encode,
    {
        self.entries.insert(key, value.encode());
    }

    /// Loads the value under the given key if any.
    pub fn load<T>(&self, key: &[u8]) -> Option<Result<T>>
    where
        T: scale::Decode,
    {
        self.entries.get(key).map(|encoded| {
            <T as scale::Decode>::decode(&mut &encoded[..]).map_err(Into::into)
        })
    }
}
