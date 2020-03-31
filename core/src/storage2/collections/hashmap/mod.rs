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

use crate::{
    hash::{
        hasher::{
            Blake2x256Hasher,
            Hasher,
        },
        HashBuilder,
    },
    storage2::{
        LazyHashMap,
        Stash,
    },
};
use core::{
    borrow::Borrow,
    cmp::Eq,
};
use ink_prelude::vec::Vec;
use ink_primitives::Key;

/// The index type within a hashmap.
///
/// # Note
///
/// Used for key indices internal to the hashmap.
pub type Index = u32;

pub struct HashMap<K, V, H = Blake2x256Hasher>
where
    H: Hasher,
{
    /// The keys of the storage hash map.
    keys: Stash<K>,
    /// The values of the storage hash map.
    values: LazyHashMap<K, ValueEntry<V>, H>,
}

/// An entry within the storage hash map.
///
/// Stores the value as well as the index to its associated key.
#[derive(Debug, scale::Encode, scale::Decode)]
pub struct ValueEntry<V> {
    /// The value stored in this entry.
    value: V,
    /// The index of the key associated with this value.
    key_index: Index,
}

/// The key pair that is used to compute the actual mapping storage offsets.
///
/// # Note
///
/// The storage hashmap calculates its storage access by hashing its storage
/// key and the encoded bytes of the given value key and hashing it with the
/// associated hasher. The `[u8; 32]` result is then converted into a storage
/// key with which the contract storage is accessed through `LazyMapping`.
#[derive(Debug, scale::Encode)]
pub struct KeyPair<'a, 'b, K> {
    /// The storage hashmap's storage key.
    storage_key: &'a Key,
    /// The key associated with the queried value.
    value_key: &'b K,
}

impl<K, V, H> HashMap<K, V, H>
where
    K: Ord,
    H: Hasher,
{
    /// Creates a new empty storage hash map.
    pub fn new() -> Self {
        Self {
            keys: Stash::new(),
            values: LazyHashMap::new(),
        }
    }

    /// Returns the number of key- value pairs stored in the hash map.
    pub fn len(&self) -> u32 {
        self.keys.len()
    }

    /// Returns `true` if the hash map is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

impl<K, V, H> HashMap<K, V, H>
where
    H: Hasher,
{
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// # Note
    ///
    /// - If the map did have this key present, the value is updated,
    ///   and the old value is returned. The key is not updated, though;
    ///   this matters for types that can be `==` without being identical.
    pub fn insert(&mut self, key: K, new_value: V) -> Option<V> {
        todo!()
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// # Note
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but `Hash` and `Eq` on the borrowed form must match those for the key type.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: scale::Encode + Eq + ?Sized,
    {
        todo!()
    }

    /// Returns a shared reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but `Hash` and `Eq` on the borrowed form must match those for the key type.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: scale::Encode + Eq + ?Sized,
    {
        todo!()
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but `Hash` and `Eq` on the borrowed form must match those for the key type.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: scale::Encode + Eq + ?Sized,
    {
        todo!()
    }

    /// Returns `true` if there is an entry corresponding to the key in the map.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: scale::Encode + Eq + ?Sized,
    {
        todo!()
    }
}
