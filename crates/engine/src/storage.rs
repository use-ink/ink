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

use core::hash::Hash;
use std::{
    borrow::Borrow,
    cmp::Eq,
    collections::HashMap,
};

/// Provides the storage backend.
#[derive(Default)]
pub struct Storage<K, V> {
    hmap: HashMap<K, V>,
}

impl<K, V> Storage<K, V>
where
    K: Eq + Hash,
{
    /// Creates a new storage instance.
    pub fn new() -> Self {
        Storage {
            hmap: HashMap::new(),
        }
    }

    /// Returns the amount of entries in the storage.
    pub fn len(&self) -> usize {
        self.hmap.len()
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.hmap.get(key)
    }

    /// Removes a key from the storage, returning the value at the key if the key
    /// was previously in storage.
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
        Q: core::hash::Hash + Eq,
    {
        self.hmap.remove(key)
    }

    /// Sets the value of the entry, and returns the entry's old value.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.hmap.insert(key, value)
    }
}
