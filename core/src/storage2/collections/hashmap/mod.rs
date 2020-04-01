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

mod iter;

use self::iter::Iter;
use crate::{
    hash::hasher::{
        Blake2x256Hasher,
        Hasher,
    },
    storage2::{
        LazyHashMap,
        Pack,
        PullAt,
        PullForward,
        Stash,
        StorageFootprint,
    },
};
use core::{
    borrow::Borrow,
    cmp::Eq,
};
use ink_primitives::Key;

/// The index type within a hashmap.
///
/// # Note
///
/// Used for key indices internal to the hashmap.
type KeyIndex = u32;

pub struct HashMap<K, V, H = Blake2x256Hasher>
where
    H: Hasher,
{
    /// The keys of the storage hash map.
    keys: Stash<K>,
    /// The values of the storage hash map.
    values: LazyHashMap<K, Pack<ValueEntry<V>>, H>,
}

/// An entry within the storage hash map.
///
/// Stores the value as well as the index to its associated key.
#[derive(Debug, scale::Encode, scale::Decode)]
pub struct ValueEntry<V> {
    /// The value stored in this entry.
    value: V,
    /// The index of the key associated with this value.
    key_index: KeyIndex,
}

impl<V> PullAt for ValueEntry<V>
where
    V: scale::Decode,
{
    fn pull_at(at: Key) -> Self {
        crate::storage2::pull_single_cell(at)
    }
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

    /// Returns an iterator yielding shared references to all key/value pairs
    /// of the hash map.
    ///
    /// # Note
    ///
    /// - Avoid unbounded iteration over big storage stashs.
    /// - Prefer using methods like `Iterator::take` in order to limit the number
    ///   of yielded elements.
    pub fn iter(&self) -> Iter<K, V, H> {
        Iter::new(self)
    }
}

impl<K, V, H> HashMap<K, V, H>
where
    K: Ord + Eq + Clone + scale::Codec + PullForward + StorageFootprint,
    V: scale::Decode,
    H: Hasher,
    Key: From<H::Output>,
{
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// # Note
    ///
    /// - Prefer this operation over [`HashMap::insert_get`] if the return value
    ///   of the previous value associated with the same key is not required.
    /// - If the map did have this key present, the value is updated,
    ///   and the old value is returned. The key is not updated, though;
    ///   this matters for types that can be `==` without being identical.
    pub fn insert<Q>(&mut self, key: K, new_value: V) -> Option<()> {
        let key_index = self.keys.put(key.to_owned());
        self.values.put(
            key,
            Some(Pack::new(ValueEntry {
                value: new_value,
                key_index,
            })),
        );
        Some(())
    }

    /// Inserts a key-value pair into the map.
    ///
    /// Returns the previous value associated with the same key if any.
    /// If the map did not have this key present, `None` is returned.
    ///
    /// # Note
    ///
    /// - Prefer [`HashMap::insert`] if the return value of the previous value
    ///   associated with the same key is not required.
    /// - If the map did have this key present, the value is updated,
    ///   and the old value is returned. The key is not updated, though;
    ///   this matters for types that can be `==` without being identical.
    pub fn insert_get<Q>(&mut self, key: &Q, new_value: V) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + scale::Encode + ToOwned<Owned = K>,
    {
        let key_index = self.keys.put(key.to_owned());
        self.values
            .put_get(
                key,
                Some(Pack::new(ValueEntry {
                    value: new_value,
                    key_index,
                })),
            )
            .map(|entry| Pack::into_inner(entry).value)
    }

    /// Removes the key/value pair from the map associated with the given key.
    ///
    /// - Returns the removed value if any.
    /// - Prefer [`HashMap::remove`] in case the returned value is not required.
    ///
    /// # Note
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but `Hash` and `Eq` on the borrowed form must match those for the key type.
    pub fn take<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + scale::Encode + ToOwned<Owned = K>,
    {
        let entry = self.values.put_get(key, None).map(Pack::into_inner)?;
        self.keys
            .take(entry.key_index)
            .expect("`key_index` must point to a valid key entry");
        Some(entry.value)
    }

    /// Removes the key/value pair from the map associated with the given key.
    ///
    /// - Returns `Some` if there was an associated value for the given key.
    /// - Prefer this operation over [`HashMap::take`] in case the returned value
    ///   is not required.
    ///
    /// # Note
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but `Hash` and `Eq` on the borrowed form must match those for the key type.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<()>
    where
        K: Borrow<Q>,
        Q: Ord + scale::Encode + ToOwned<Owned = K>,
    {
        let entry = self.values.put_get(key, None).map(Pack::into_inner)?;
        // TODO: Add Stash::take_drop for use cases where the return value
        //       is not required to avoid reading from storage.
        self.keys
            .take(entry.key_index)
            .expect("`key_index` must point to a valid key entry");
        Some(())
    }

    /// Returns a shared reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but `Hash` and `Eq` on the borrowed form must match those for the key type.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + scale::Encode + ToOwned<Owned = K>,
    {
        self.values
            .get(key)
            .map(Pack::as_inner)
            .map(|entry| &entry.value)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but `Hash` and `Eq` on the borrowed form must match those for the key type.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + scale::Encode + ToOwned<Owned = K>,
    {
        self.values
            .get_mut(key)
            .map(Pack::as_inner_mut)
            .map(|entry| &mut entry.value)
    }

    /// Returns `true` if there is an entry corresponding to the key in the map.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + PartialEq<K> + Eq + scale::Encode + ToOwned<Owned = K>,
    {
        self.values
            .get(key)
            .map(Pack::as_inner)
            .map(|entry| entry.key_index)
            .and_then(|key_index| {
                self.keys.get(key_index).map(|stored_key| key == stored_key)
            })
            .unwrap_or(false)
    }

    /// Defragments storage used by the storage hash map.
    ///
    /// # Note
    ///
    /// This frees storage that is hold but not necessary for the hash map to hold.
    /// This operation might be expensive, especially for big `max_iteration`
    /// parameters. The `max_iterations` parameter can be used to limit the
    /// expensiveness for this operation and instead free up storage incrementally.
    pub fn defrag(&mut self, _max_iterations: Option<u32>) {
        todo!()
    }
}
