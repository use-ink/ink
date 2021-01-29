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

use super::{
    HashMap as StorageHashMap,
    Iter,
    IterMut,
};
use crate::traits::PackedLayout;
use core::{
    cmp::{
        Eq,
        Ord,
        PartialEq,
    },
    iter::FromIterator,
    ops,
};
use ink_env::hash::{
    CryptoHash,
    HashOutput,
};
use ink_prelude::borrow::{
    Borrow,
    ToOwned,
};
use ink_primitives::Key;

impl<K, V, H> Drop for StorageHashMap<K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    fn drop(&mut self) {
        self.clear_cells();
    }
}

impl<K, V, H> Default for StorageHashMap<K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, K, V, H, Q> ops::Index<&'a Q> for StorageHashMap<K, V, H>
where
    Q: Ord + scale::Encode + ToOwned<Owned = K>,
    K: Borrow<Q> + Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<'a, K, V, H, Q> ops::IndexMut<&'a Q> for StorageHashMap<K, V, H>
where
    Q: Ord + scale::Encode + ToOwned<Owned = K>,
    K: Borrow<Q> + Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    fn index_mut(&mut self, index: &Q) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

impl<'a, K: 'a, V: 'a, H> IntoIterator for &'a StorageHashMap<K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V, H>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: 'a, V: 'a, H> IntoIterator for &'a mut StorageHashMap<K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V, H>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, H> Extend<(K, V)> for StorageHashMap<K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

impl<K, V, H> FromIterator<(K, V)> for StorageHashMap<K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut vec = StorageHashMap::new();
        vec.extend(iter);
        vec
    }
}

impl<K, V, H> PartialEq for StorageHashMap<K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: PartialEq + PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false
        }
        self.iter()
            .map(|(key, value)| (value, other.get(key)))
            .all(|(lhs, maybe_rhs)| maybe_rhs.map(|rhs| rhs == lhs).unwrap_or(false))
    }
}

impl<K, V, H> Eq for StorageHashMap<K, V, H>
where
    K: Ord + Clone + PackedLayout,
    V: Eq + PackedLayout,
    H: CryptoHash,
    Key: From<<H as HashOutput>::Type>,
{
}
