// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

/// This file contains the logic and model for each node in the tree.
use crate::storage::{
    btree_map::impls::{
        KVStorageIndex,
        B,
        CAPACITY,
    },
    Flush,
};
use core::{
    marker::PhantomData,
    ops::Range,
};
use scale::{
    Decode,
    Encode,
};
#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

/// Number of edges each node has.
///
/// Note: `CAPACITY + 1 == EDGES` must always be true!
const EDGES: usize = 2 * B;

/// Reference to a key/value pair in the tree.
#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub(super) struct KVPair<K, V> {
    /// A key.
    key: K,
    /// A value.
    value: V,
}

impl<K, V> KVPair<K, V> {
    /// Creates a new `KVPair` from a `key` and a `value`.
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }

    /// Returns a tuple of (`key`, `value`) stored in this key/value pair.
    pub(super) fn kv(self) -> (K, V) {
        (self.key, self.value)
    }

    /// Returns a reference to the `key` stored in this key/value pair.
    pub(super) fn key_ref(&self) -> &K {
        &self.key
    }

    /// Returns the `value` stored in this key/value pair.
    pub(super) fn value(self) -> V {
        self.value
    }

    /// Returns a mutable reference to the `value` stored in this key/value pair.
    pub(super) fn value_ref_mut(&mut self) -> &mut V {
        &mut self.value
    }

    /// Returns a reference to the `value` stored in this key/value pair.
    pub(super) fn value_ref(&self) -> &V {
        &self.value
    }
}

impl<K, V> Flush for KVPair<K, V>
where
    K: Encode + Flush,
    V: Encode + Flush,
{
    #[inline]
    fn flush(&mut self) {
        self.key.flush();
        self.value.flush();
    }
}

/// Reference to a key/value pair in the tree.
pub(super) struct KVRef<'a, K, V> {
    /// Reference to the key.
    key: &'a K,
    /// Reference to the value.
    value: &'a V,
}

impl<'a, K, V> KVRef<'a, K, V> {
    /// Creates a `KVRef` from a `KVPair`.
    pub(super) fn new(pair: &'a KVPair<K, V>) -> Self {
        Self {
            key: pair.key_ref(),
            value: pair.value_ref(),
        }
    }

    /// Returns a tuple with references to the `key` and `value`
    /// stored in this key/value pair..
    pub(super) fn kv(self) -> (&'a K, &'a V) {
        (self.key, self.value)
    }

    /// Returns a reference to the `key` stored in this key/value pair.
    pub(super) fn key(self) -> &'a K {
        self.key
    }
}

/// A node in the tree.
///
/// Each node contains `CAPACITY` keys and values and an edges array over
/// which children nodes can be linked. Each child has a link to its parent.
///
/// Each node is stored as one storage entity. This reduces storage access,
/// since with each fetch the entire content of a node (all its elements, etc.)
/// are fetched.
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub(super) struct Node<K, V> {
    /// A reference to this node's parent node.
    parent: Option<NodeHandle>,

    /// This node's index into the parent node's `edges` array.
    /// If, for example, `parent_idx = Some(2)` this refers to the
    /// second position in the `edges` array of its parent node.
    parent_idx: Option<u32>,

    /// The array storing the storage index of the key/value pairs in a node.
    /// For performance reasons each node contains only the index where the
    /// key/value pair is stored in `kv_pairs`. Otherwise re-balancing the tree
    /// or moving entries would be very expensive. With this indirection we only
    /// have to move the `u32` index.
    pub(super) pairs: [Option<KVStorageIndex>; CAPACITY],

    /// The pointers to the children of this node.
    pub(super) edges: [Option<NodeHandle>; EDGES],

    /// Number of elements stored in this node.
    len: u32,

    /// Marker for compile-time checking of correct key/value pair types.
    marker: PhantomData<(K, V)>,
}

impl<K, V> Flush for Node<K, V>
where
    K: Encode + Flush,
    V: Encode + Flush,
{
    #[inline]
    fn flush(&mut self) {
        self.parent.flush();
        self.parent_idx.flush();
        self.pairs.flush();
        self.edges.flush();
        self.len.flush();
    }
}

impl<K, V> Node<K, V> {
    /// Create a new `Node`. The node is empty and all fields are instantiated with `None`.
    pub(super) fn new() -> Self {
        Node {
            parent: None,
            parent_idx: None,
            pairs: Default::default(),
            edges: [None; 2 * B],
            len: 0,
            marker: Default::default(),
        }
    }

    /// Returns the number of elements (i.e. key/value pairs) stored in this node.
    pub(super) fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns a reference to the edges (i.e. the children) stored in this node.
    #[cfg(test)]
    pub(super) fn edges(&self) -> &[Option<NodeHandle>; EDGES] {
        &self.edges
    }

    /// Sets a new `edge` at `idx`.
    pub(super) fn set_edge(&mut self, idx: usize, edge: Option<NodeHandle>) {
        self.edges[idx] = edge;
    }

    /// Takes the edge at `idx` and returns it. Puts `None` at `idx` instead then.
    pub(super) fn take_edge(&mut self, idx: usize) -> Option<NodeHandle> {
        self.edges[idx].take()
    }

    /// Sets a supplied range of `edges` to the ones from the supplied `src` slice.
    pub(super) fn set_edges_range(
        &mut self,
        src: &[Option<NodeHandle>],
        range: Range<usize>,
    ) {
        self.edges[range].copy_from_slice(src);
    }

    /// Returns a reference to the key/value `pairs` stored in this node.
    pub(super) fn pairs(&self) -> &[Option<KVStorageIndex>; CAPACITY] {
        &self.pairs
    }

    /// Pushes the storage index of a key/value pair into the `pairs` array of this
    /// node while shifting all subsequent items to the right by one. The last element
    /// of the slice will fall out.
    pub(super) fn push_pair_to(&mut self, idx: usize, pair_index: KVStorageIndex) {
        slice_insert(&mut self.pairs, idx, Some(pair_index));
    }

    /// Removes the storage index at `idx` from `pairs` while shifting all subsequent items to
    /// the left by one.
    ///
    /// Returns the removed storage index.
    pub(super) fn remove_pair_with_shift(
        &mut self,
        idx: usize,
    ) -> Option<KVStorageIndex> {
        slice_remove(&mut self.pairs, idx)
    }

    /// Returns a reference to the storage index of the key/value pair stored
    /// at `idx` in this node. `idx` refers to the position of the pair within
    /// the `pairs` array.
    pub(super) fn pair(&self, idx: usize) -> &Option<KVStorageIndex> {
        &self.pairs[idx]
    }

    /// Sets a new `parent` for this node.
    pub(super) fn set_parent(&mut self, parent: Option<NodeHandle>) {
        self.parent = parent;
    }

    /// Sets a new `parent_idx` for this node.
    /// `parent_idx` refers to the position which the parent node has
    /// within the `edges` array of the node behind `self.parent`.
    pub(super) fn set_parent_idx(&mut self, parent_idx: Option<usize>) {
        self.parent_idx = parent_idx.map(|p| p as u32);
    }

    /// Sets a new `pair_storage_index` at `idx`. Returns the old one at this `idx`, if existent.
    pub(super) fn set_pair(
        &mut self,
        idx: usize,
        pair_storage_index: Option<KVStorageIndex>,
    ) -> Option<KVStorageIndex> {
        match pair_storage_index {
            Some(pair_index) => self.pairs[idx].replace(pair_index),
            None => self.pairs[idx].take(),
        }
    }

    /// Takes the pair at `idx` and returns it. Puts `None` at `idx` instead then.
    pub(super) fn take_pair(&mut self, idx: usize) -> Option<KVStorageIndex> {
        self.pairs[idx].take()
    }

    /// Sets a supplied range of `pairs` to the ones from the supplied `src` slice.
    pub(super) fn set_pair_range(
        &mut self,
        src: &[Option<KVStorageIndex>],
        range: Range<usize>,
    ) {
        self.pairs[range].copy_from_slice(src);
    }

    /// Returns the parent of the current node, if available.
    pub(super) fn parent(&self) -> Option<NodeHandle> {
        self.parent
    }

    /// Returns the number of edges (i.e. children) this node has.
    pub(super) fn edges_count(&self) -> usize {
        self.edges.iter().filter(|o| o.is_some()).count()
    }

    /// Returns the edge at position `idx`.
    pub(super) fn edge(&self, idx: usize) -> Option<NodeHandle> {
        self.edges[idx]
    }

    /// Pushes an edge into the `edges` array of this node while shifting all
    /// subsequent items to the right by one. The last element of the slice
    /// will fall out.
    pub(super) fn push_edge_to(&mut self, idx: usize, edge: NodeHandle) {
        slice_insert(&mut self.edges, idx, Some(edge));
    }

    /// Removes the element at `idx` from `edges` while shifting all subsequent items to
    /// the left by one.
    ///
    /// Returns the removed element.
    pub(super) fn remove_edge_with_shift(&mut self, idx: usize) -> Option<NodeHandle> {
        slice_remove(&mut self.edges, idx)
    }

    /// Returns the position of this node in the parent node's `edges` array.
    pub(super) fn parent_idx(&self) -> Option<usize> {
        self.parent_idx.map(|v| v as usize)
    }

    /// Returns the number of edges (i.e. children) this node has.
    pub(super) fn set_len(&mut self, new_len: usize) {
        self.len = new_len as u32;
    }
}

/// Points to a node in the tree.
#[derive(Debug, Clone, Copy, Encode, Decode, PartialEq, Eq)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub(super) struct NodeHandle {
    node: u32,
}

impl Flush for NodeHandle {
    #[inline]
    fn flush(&mut self) {
        self.node.flush();
    }
}

impl NodeHandle {
    /// Create a new `NodeHandle` from a `u32` which must be the index
    /// this node has in the `entries` storage.
    pub(super) fn new(node: u32) -> Self {
        Self { node }
    }

    /// Returns the node index for this node.
    /// The index must point to the nodes position in the `entries` storage.
    pub(super) fn node(self) -> u32 {
        self.node
    }
}

/// Points to a specific key/value pair within a node in the tree.
#[derive(Clone, Copy)]
pub(super) struct KVHandle {
    /// Index of the node in the `entries` storage.
    node: NodeHandle,
    /// Index of the key/value pair within the node. This is a pointer
    /// to the position in the `kv_pairs` array.
    idx: u32,
}

impl KVHandle {
    /// Creates a new `KVHandle` from a `NodeHandle` and an `idx`.
    /// The `idx` is the node's index in the parent node's `edges` array.
    pub(super) fn new(node: NodeHandle, idx: usize) -> Self {
        Self {
            node,
            idx: idx as u32,
        }
    }

    /// Returns the `NodeHandle` for this node.
    /// The index should point to the nodes position in the `entries` storage.
    pub(super) fn node(self) -> NodeHandle {
        self.node
    }

    /// Returns the position in the parent node's `edges` array to which
    /// this handle points.
    pub(super) fn idx(self) -> usize {
        self.idx as usize
    }
}

/// Inserts `val` at `idx` into `slice` while shifting all subsequent items to
/// the right by one. The last element of the slice will fall out.
pub fn slice_insert<T: Copy>(slice: &mut [T], i: usize, val: T) {
    let len = slice.len();
    let from = i;
    let to = i + 1;
    slice.copy_within(from..len - 1, to);

    slice[i] = val;
}

/// Extracts the element at `idx` from `slice` while shifting all subsequent items to
/// the left by one.
///
/// Returns the extracted element.
fn slice_remove<T: Copy>(slice: &mut [Option<T>], i: usize) -> Option<T> {
    let len = slice.len();
    let val = slice[i].take();
    let from = i + 1;
    let to = i;

    slice.copy_within(from.., to);
    slice[len - 1] = None;
    val
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_insert_works() {
        // given
        let mut sl = [Some(1), Some(2), Some(3), Some(4)];

        // when
        slice_insert(&mut sl, 2, Some(99));

        // then
        assert_eq!(sl, [Some(1), Some(2), Some(99), Some(3)]);
    }

    #[test]
    fn slice_remove_works() {
        // given
        let mut sl = [Some(1), Some(2), Some(3), Some(4)];

        // when
        let removed = slice_remove(&mut sl, 2);

        // then
        assert_eq!(removed, Some(3));
        assert_eq!(sl, [Some(1), Some(2), Some(4), None]);
    }
}
