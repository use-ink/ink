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

use super::ValueEntry;
use crate::{
    collections::{
        extend_lifetime,
        stash::Iter as StashIter,
        HashMap as StorageHashMap,
    },
    lazy::LazyHashMap,
    traits::PackedLayout,
};
use ink_env::hash::{
    CryptoHash,
    HashOutput,
};
use ink_primitives::Key;

/// An iterator over shared references to the elements of a storage hash map.
#[derive(Debug, Copy, Clone)]
pub struct Iter<'a, K, V, H>
where
    K: PackedLayout,
{
    /// The iterator over the map's keys.
    keys_iter: StashIter<'a, K>,
    /// The lazy hash map to query the values.
    values: &'a LazyHashMap<K, ValueEntry<V>, H>,
}

impl<'a, K, V, H> Iter<'a, K, V, H>
where
    K: Ord + Clone + PackedLayout,
{
    /// Creates a new iterator for the given storage hash map.
    pub(crate) fn new(hash_map: &'a StorageHashMap<K, V, H>) -> Self
    where
        V: PackedLayout,
        H: CryptoHash,
        Key: From<<H as HashOutput>::Type>,
    {
        Self {
            keys_iter: hash_map.keys.iter(),
            values: &hash_map.values,
        }
    }
}

impl<'a, K, V, H> Iter<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    /// Queries the value for the given key and returns the key/value pair.
    ///
    /// # Panics
    ///
    /// If the key refers to an invalid element.
    fn query_value(&self, key: &'a K) -> <Self as Iterator>::Item {
        let entry = self
            .values
            .get(key)
            .expect("a key must always refer to an existing entry");
        (key, &entry.value)
    }
}

impl<'a, K, V, H> Iterator for Iter<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    type Item = (&'a K, &'a V);

    fn count(self) -> usize {
        self.keys_iter.count()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.keys_iter.next()?;
        Some(self.query_value(key))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.keys_iter.size_hint()
    }
}

impl<'a, K, V, H> ExactSizeIterator for Iter<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
}

impl<'a, K, V, H> DoubleEndedIterator for Iter<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let key = self.keys_iter.next_back()?;
        Some(self.query_value(key))
    }
}

/// An iterator over shared references to the elements of a storage hash map.
#[derive(Debug)]
pub struct IterMut<'a, K, V, H>
where
    K: PackedLayout,
{
    /// The iterator over the map's keys.
    keys_iter: StashIter<'a, K>,
    /// The lazy hash map to query the values.
    values: &'a mut LazyHashMap<K, ValueEntry<V>, H>,
}

impl<'a, K, V, H> IterMut<'a, K, V, H>
where
    K: Ord + Clone + PackedLayout,
{
    /// Creates a new iterator for the given storage hash map.
    pub(crate) fn new(hash_map: &'a mut StorageHashMap<K, V, H>) -> Self
    where
        V: PackedLayout,
        H: CryptoHash,
        Key: From<<H as HashOutput>::Type>,
    {
        Self {
            keys_iter: hash_map.keys.iter(),
            values: &mut hash_map.values,
        }
    }
}

impl<'a, K, V, H> IterMut<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    /// Queries the value for the given key and returns the key/value pair.
    ///
    /// # Panics
    ///
    /// If the key refers to an invalid element.
    fn query_value<'b>(&'b mut self, key: &'a K) -> <Self as Iterator>::Item {
        let entry = self
            .values
            .get_mut(key)
            .expect("a key must always refer to an existing entry");
        (key, unsafe {
            extend_lifetime::<'b, 'a, V>(&mut entry.value)
        })
    }
}

impl<'a, K, V, H> Iterator for IterMut<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    type Item = (&'a K, &'a mut V);

    fn count(self) -> usize {
        self.keys_iter.count()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.keys_iter.next()?;
        Some(self.query_value(key))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.keys_iter.size_hint()
    }
}

impl<'a, K, V, H> ExactSizeIterator for IterMut<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
}

impl<'a, K, V, H> DoubleEndedIterator for IterMut<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let key = self.keys_iter.next_back()?;
        Some(self.query_value(key))
    }
}

/// An iterator over shared references to the values of a storage hash map.
#[derive(Debug, Copy, Clone)]
pub struct Values<'a, K, V, H>
where
    K: PackedLayout,
{
    /// The key/values pair iterator.
    iter: Iter<'a, K, V, H>,
}

impl<'a, K, V, H> Values<'a, K, V, H>
where
    K: Ord + Clone + PackedLayout,
{
    /// Creates a new iterator for the given storage hash map.
    pub(crate) fn new(hash_map: &'a StorageHashMap<K, V, H>) -> Self
    where
        V: PackedLayout,
        H: CryptoHash,
        Key: From<<H as HashOutput>::Type>,
    {
        Self {
            iter: hash_map.iter(),
        }
    }
}

impl<'a, K, V, H> Iterator for Values<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    type Item = &'a V;

    fn count(self) -> usize {
        self.iter.count()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_key, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V, H> ExactSizeIterator for Values<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
}

impl<'a, K, V, H> DoubleEndedIterator for Values<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(_key, value)| value)
    }
}

/// An iterator over exclusive references to the values of a storage hash map.
#[derive(Debug)]
pub struct ValuesMut<'a, K, V, H>
where
    K: PackedLayout,
{
    /// The key/values pair iterator.
    iter: IterMut<'a, K, V, H>,
}

impl<'a, K, V, H> ValuesMut<'a, K, V, H>
where
    K: Ord + Clone + PackedLayout,
{
    /// Creates a new iterator for the given storage hash map.
    pub(crate) fn new(hash_map: &'a mut StorageHashMap<K, V, H>) -> Self
    where
        V: PackedLayout,
        H: CryptoHash,
        Key: From<<H as HashOutput>::Type>,
    {
        Self {
            iter: hash_map.iter_mut(),
        }
    }
}

impl<'a, K, V, H> Iterator for ValuesMut<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    type Item = &'a mut V;

    fn count(self) -> usize {
        self.iter.count()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_key, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V, H> ExactSizeIterator for ValuesMut<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
}

impl<'a, K, V, H> DoubleEndedIterator for ValuesMut<'a, K, V, H>
where
    K: Ord + Eq + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(_key, value)| value)
    }
}

/// An iterator over references to the keys of a storage hash map.
#[derive(Debug, Copy, Clone)]
pub struct Keys<'a, K>
where
    K: PackedLayout,
{
    /// The key iterator.
    iter: StashIter<'a, K>,
}

impl<'a, K> Keys<'a, K>
where
    K: Ord + Clone + PackedLayout,
{
    /// Creates a new iterator for the given storage hash map.
    pub(crate) fn new<V, H>(hash_map: &'a StorageHashMap<K, V, H>) -> Self
    where
        V: PackedLayout,
        H: CryptoHash,
        Key: From<<H as HashOutput>::Type>,
    {
        Self {
            iter: hash_map.keys.iter(),
        }
    }
}

impl<'a, K> Iterator for Keys<'a, K>
where
    K: PackedLayout,
{
    type Item = &'a K;

    fn count(self) -> usize {
        self.iter.count()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K> ExactSizeIterator for Keys<'a, K> where K: PackedLayout {}

impl<'a, K> DoubleEndedIterator for Keys<'a, K>
where
    K: PackedLayout,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}
