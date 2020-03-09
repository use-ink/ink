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

use self::{
    HandleType::{
        Branch,
        Leaf,
    },
    InsertResult::{
        Fit,
        Split,
    },
};
use super::search::{
    self,
    SearchResult::{
        Found,
        NotFound,
    },
};
use crate::storage::{
    self,
    alloc::{
        Allocate,
        AllocateUsing,
        Initialize,
    },
    btree_map::node::{
        KVHandle,
        KVPair,
        KVRef,
        Node,
        NodeHandle,
    },
    chunk::SyncChunk,
    Flush,
};
use core::{
    borrow::Borrow,
    cmp::Ord,
};
#[cfg(feature = "ink-generate-abi")]
use ink_abi::{
    HasLayout,
    LayoutField,
    LayoutStruct,
    StorageLayout,
};
use scale::{
    Codec,
    Decode,
    Encode,
};
#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

/// Each node in the tree has 2 * B children.
///
/// For the moment we use the same number as the Rust standard library
/// implementation here. There is no empiric evidence for the effectiveness
/// of this number yet -- we just chose this number to have a first starting
/// point.
pub(super) const B: usize = 6;

/// Number of elements (i.e. key/value pairs) which can be stored in one
/// node of the tree. The `- 1` is because there needs to be an edge to
/// the right of the last element.
///
/// ```no_compile
/// const B: usize = 2;
/// const CAPACITY: usize = 2 * B - 1;
/// keys  = [    a,    b,    c    ];
/// edges = [ 1,    2,    3,    4 ];
/// ```
pub(super) const CAPACITY: usize = 2 * B - 1;

/// The index where a key/value pair is stored in `kv_pairs`.
pub(super) type KVStorageIndex = u32;

/// The node type, either a `Leaf` (a node without children) or `Branch`
/// (a node with children). This distinction makes it easier to handle e.g.
/// cases where nodes are merged or implementing recursion.
pub(super) enum HandleType {
    Leaf,
    Branch,
}

/// This enum is used when recursively processing an underfull node (i.e. a node has so
/// few elements that it could be merged with another one). This happens when a key/value
/// pair is removed from a node.
///
/// This enum is returned as a result of `.handle_underfull_node`, which contains
/// the underfull handling logic. Based on this result the recursion either proceeds or stops.
enum UnderflowResult {
    /// We are at the root now.
    AtRoot,
    /// The node has an empty parent, referred to by the `NodeHandle` in this tuple.
    EmptyParent(NodeHandle),
    /// The elements of an underfull node were merged into its neighboring child.
    /// `NodeHandle` points to the parent node of these children nodes.
    /// The node which was merged was subsequently removed.
    Merged(NodeHandle),
    /// If merging is not possible we "steal" an element from one child and put it into
    /// its neighboring child. `NodeHandle` is the parent node.
    Stole(NodeHandle),
}

/// Mapping stored in the contract storage.
///
/// This implementation follows the algorithm used by the Rust
/// `BTreeMap` standard library implementation. The Rust implementation
/// is in-memory, whereas this implementation uses the ink! storage
/// primitives (`SyncChunk`, etc.).
#[derive(Debug)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct BTreeMap<K, V> {
    /// Stores densely packed general BTreeMap information.
    header: storage::Value<BTreeMapHeader>,

    /// Contains the nodes of the tree.
    nodes: SyncChunk<InternalEntry<K, V>>,

    /// The key/value pairs stored in the tree.
    /// We don't store this in `nodes` for performance reasons. Instead each
    /// node of the tree merely stores the storage index where the key/value
    /// pair is stored.
    /// This way it's more effective to balance the tree, since we only ever
    /// have to move the `u32` index, instead of the whole `K`/`V` (which
    /// would imply decoding, encoding, etc.).
    kv_pairs: SyncChunk<InternalKVEntry<K, V>>,
}

/// Densely stored general information required by a map.
///
/// # Note
///
/// Separation of these fields into a sub structure has been made
/// for performance reasons so that they all reside in the same
/// storage entity. This allows implementations to perform less reads
/// and writes to the underlying contract storage.
#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
struct BTreeMapHeader {
    /// The latest vacant node index.
    next_vacant: Option<NodeHandle>,
    /// The latest vacant pair index.
    next_vacant_pair: Option<KVStorageIndex>,
    /// The index of the root node.
    root: Option<NodeHandle>,
    /// The number of elements stored in the map.
    ///
    /// # Note
    ///
    /// We cannot simply use the underlying length of the vector
    /// since it would include vacant slots as well.
    len: u32,
    /// Number of nodes the tree contains. This is not the number
    /// of elements, since each node may contain multiple elements
    /// (i.e. key/value pairs)!
    node_count: u32,
}

impl Flush for BTreeMapHeader {
    #[inline]
    fn flush(&mut self) {
        self.next_vacant.flush();
        self.next_vacant_pair.flush();
        self.root.flush();
        self.len.flush();
        self.node_count.flush();
    }
}

impl<K, V> Initialize for BTreeMap<K, V> {
    type Args = ();

    #[inline(always)]
    fn default_value() -> Option<Self::Args> {
        Some(())
    }

    #[inline]
    fn initialize(&mut self, _args: Self::Args) {
        self.header.set(BTreeMapHeader {
            next_vacant: None,
            next_vacant_pair: None,
            len: 0,
            node_count: 0,
            root: None,
        });
    }
}

impl<K: Ord, V> BTreeMap<K, V> {
    /// Returns the number of elements stored in the map.
    pub fn len(&self) -> u32 {
        self.header.len
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V> Flush for BTreeMap<K, V>
where
    K: Encode + Flush,
    V: Encode + Flush,
{
    #[inline]
    fn flush(&mut self) {
        self.header.flush();
        self.nodes.flush();
        self.kv_pairs.flush();
    }
}

impl<K, V> Encode for BTreeMap<K, V> {
    fn encode_to<W: scale::Output>(&self, dest: &mut W) {
        self.header.encode_to(dest);
        self.nodes.encode_to(dest);
        self.kv_pairs.encode_to(dest);
    }
}

impl<K, V> Decode for BTreeMap<K, V> {
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        let header = storage::Value::decode(input)?;
        let entries = SyncChunk::decode(input)?;
        let kv_pairs = SyncChunk::decode(input)?;
        Ok(Self {
            header,
            nodes: entries,
            kv_pairs,
        })
    }
}

impl<K, V> AllocateUsing for BTreeMap<K, V> {
    #[inline]
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            header: storage::Value::allocate_using(alloc),
            nodes: SyncChunk::allocate_using(alloc),
            kv_pairs: SyncChunk::allocate_using(alloc),
        }
    }
}

impl<K, V> Extend<(K, V)> for BTreeMap<K, V>
where
    K: Codec + Ord,
    V: Codec,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<'a, K, V> Extend<(&'a K, &'a V)> for BTreeMap<K, V>
where
    K: Codec + Ord + Copy,
    V: Codec + Copy,
{
    fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: I) {
        self.extend(iter.into_iter().map(|(&key, &value)| (key, value)));
    }
}

impl<'a, K, V> Extend<&'a (K, V)> for BTreeMap<K, V>
where
    K: Codec + Ord + Copy + 'a,
    V: Codec + Copy + 'a,
{
    fn extend<I: IntoIterator<Item = &'a (K, V)>>(&mut self, iter: I) {
        self.extend(iter.into_iter().copied());
    }
}

impl<K, V> BTreeMap<K, V> {
    /// Returns the index of the root node.
    pub(super) fn root(&self) -> &Option<NodeHandle> {
        &self.header.root
    }
}

impl<K, V> BTreeMap<K, V>
where
    K: Codec + Ord,
    V: Codec,
{
    /// Returns the `HandleType` of `handle`. Either `Leaf` or `Branch`.
    /// A branch node always has children, whereas a leaf doesn't.
    pub(super) fn get_handle_type(&self, handle: NodeHandle) -> HandleType {
        let children = self
            .get_node(handle)
            .expect("node must exist")
            .edges_count();
        if children == 0 {
            Leaf
        } else {
            Branch
        }
    }

    /// Returns the number of nodes which this tree consists of.
    pub(super) fn node_count(&self) -> u32 {
        self.header.node_count
    }

    /// Descends to the child node pointed to by this `handle`, if one is available.
    pub(super) fn descend(&self, handle: KVHandle) -> Option<NodeHandle> {
        let node = self
            .get_node(handle.node())
            .expect("node to descend from must exist");
        node.edge(handle.idx())
    }

    /// Returns a reference to the node behind `handle`, if existent.
    ///
    /// *Note*
    /// In practice this should never return `None` (except for tests which
    /// explicitly test this). The calling side should apply an `.expect()`
    /// to the return value and proof why this can never happen.
    pub(super) fn get_node(&self, handle: NodeHandle) -> Option<&Node<K, V>> {
        let entry = self.nodes.get(handle.node())?;
        match entry {
            InternalEntry::Occupied(occupied) => Some(occupied),
            InternalEntry::Vacant(_) => None,
        }
    }

    /// Returns the keys stored in this node.
    pub(super) fn keys_in_node(&self, node: NodeHandle) -> [Option<&K>; CAPACITY] {
        let node = self.get_node(node).expect("node must exist");
        let mut ks: [Option<&K>; CAPACITY] = Default::default();

        node.pairs()
            .iter()
            .enumerate()
            .for_each(|(n, maybe_kv_storage_index)| {
                ks[n] = match maybe_kv_storage_index {
                    None => None,
                    Some(kv_storage_index) => {
                        self.get_kv_ref(*kv_storage_index)
                            .map(|pair| pair.key_ref())
                    }
                };
            });
        ks
    }

    /// Returns a reference to the storage entry behind `storage_index`, if existent.
    fn get_kv_ref(&self, storage_index: KVStorageIndex) -> Option<&KVPair<K, V>> {
        let entry = self.kv_pairs.get(storage_index)?;
        match entry {
            InternalKVEntry::Occupied(occupied) => Some(occupied),
            InternalKVEntry::Vacant(_) => None,
        }
    }

    /// Returns a mutable reference to the storage entry behind `storage_index`, if existent.
    fn get_value_mut_ref(&mut self, storage_index: KVStorageIndex) -> Option<&mut V> {
        let entry = self.kv_pairs.get_mut(storage_index)?;
        match entry {
            InternalKVEntry::Occupied(occupied) => Some(occupied.value_ref_mut()),
            InternalKVEntry::Vacant(_) => None,
        }
    }

    /// Returns a reference to the key/value pair referenced by `handle`, if available.
    fn get_kv(&self, handle: KVHandle) -> Option<KVRef<K, V>> {
        let node = self.get_node(handle.node()).expect("node must exist");
        let key_index = node.pair(handle.idx()).as_ref()?;
        let pair = self.get_kv_ref(*key_index)?;
        Some(KVRef::new(pair))
    }

    /// Returns the value referenced by `handle`, if available.
    fn get_value(&self, handle: KVHandle) -> Option<&V> {
        let node = self.get_node(handle.node()).expect("node must exist");
        let value_index = node.pair(handle.idx()).as_ref()?;
        self.get_kv_ref(*value_index).map(|pair| pair.value_ref())
    }

    /// If a parent node is set for the node referenced by `handle`, a handle to
    /// this parent node will be returned.
    fn ascend(&self, handle: NodeHandle) -> Option<KVHandle> {
        let node = self
            .get_node(handle)
            .expect("node to ascend from must exist");

        node.parent().map(|parent| {
            let idx = node
                .parent_idx()
                .expect("if parent exists, parent_idx always exist as well; qed");
            KVHandle::new(parent, idx)
        })
    }

    /// Creates a root node with `key` and `val`.
    ///
    /// Returns the storage index of the inserted pair.
    fn create_root(&mut self, key: K, val: V) -> KVStorageIndex {
        debug_assert!(self.is_empty());
        debug_assert!(self.root().is_none());

        let pair = KVPair::<K, V>::new(key, val);
        let storage_index = self.put_pair(pair);

        let mut node = Node::<K, V>::new();
        node.set_pair(0, Some(storage_index));
        node.set_len(1);

        let index = self.put(node);
        self.header.len = 1;
        self.header.root = Some(index);

        storage_index
    }

    /// Inserts `key` and `val` at `handle`.
    ///
    /// Returns a reference to the inserted value.
    fn insert_kv(&mut self, handle: KVHandle, key: K, val: V) -> KVStorageIndex {
        let mut ins_pair_index;
        let mut ins_edge;
        let out_storage_index;

        let pair = KVPair::<K, V>::new(key, val);
        let pair_storage_index = self.put_pair(pair);

        let mut cur_parent = match self.insert_into_node(handle, pair_storage_index) {
            (Fit(_), idx) => return idx,
            (Split(left, pair_index, right), storage_index) => {
                ins_pair_index = pair_index;
                ins_edge = right;
                out_storage_index = storage_index;
                self.ascend(left.node())
            }
        };

        // This loop is run when the insert operation above resulted in a split node and
        // we now have an element `ins_pair_index` which needs to be inserted at another
        // place in the tree.
        //
        // The loop tries to recursively insert the element up the tree. This might well
        // result in more split nodes with other `ins_pair_index` which then need to be
        // inserted further up.
        //
        // The loop always breaks because if the root is reached and we still have an
        // element to insert, we just create a new root with the old root as a child and
        // insert the element into the new root then.
        loop {
            match cur_parent {
                Some(parent) => {
                    match self.insert_into_branch_node(parent, ins_pair_index, ins_edge) {
                        Fit(_) => {
                            self.header.len += 1;
                            return out_storage_index
                        }
                        Split(left, storage_index, right) => {
                            ins_pair_index = storage_index;
                            ins_edge = right;
                            cur_parent = self.ascend(left.node());
                        }
                    }
                }
                None => {
                    let new_root = self.root_push_level();
                    self.header.len += 1;
                    self.push_branch(new_root, ins_pair_index, ins_edge);
                    return out_storage_index
                }
            }
        }
    }

    /// Traverses downwards from `handle`, always taking the first edge down.
    /// Once a leaf is reached a handle to the first edge in the leaf is returned.
    fn first_leaf_edge(&self, mut handle: NodeHandle) -> KVHandle {
        loop {
            match self.get_handle_type(handle) {
                Leaf => return self.first_edge(handle),
                Branch => {
                    let first_edge = self.first_edge(handle);
                    handle = self
                        .descend(first_edge)
                        .expect("every branch node has children; qed");
                }
            }
        }
    }

    /// Returns a handle to the first edge in the node.
    fn first_edge(&self, handle: NodeHandle) -> KVHandle {
        KVHandle::new(handle, 0)
    }

    /// Returns the edge left to `handle`.
    fn left_edge(&self, handle: KVHandle) -> KVHandle {
        KVHandle::new(handle.node(), handle.idx())
    }

    /// Returns the edge right to `handle`.
    fn right_edge(&self, handle: KVHandle) -> KVHandle {
        KVHandle::new(handle.node(), handle.idx() + 1)
    }

    /// Returns a handle to the key/value pair storage index left of `handle`.
    ///
    /// If `handle` already points to the first element in the node
    /// an `Err(handle)` is returned. The handle contained in `Err(handle)`
    /// is the one supplied as the `handle` argument to this function.
    fn left_kv(&self, handle: KVHandle) -> Option<KVHandle> {
        if handle.idx() > 0 {
            Some(KVHandle::new(handle.node(), handle.idx() - 1))
        } else {
            None
        }
    }

    /// Returns a handle to the key/value pair storage index right of `handle`.
    ///
    /// If `handle` already points to the last element in the node an
    /// `Err(handle)` is returned. The handle contained in `Err(handle)`
    /// is the one supplied as the `handle` argument to this function.
    fn right_kv(&self, handle: KVHandle) -> Option<KVHandle> {
        let node = self
            .get_node(handle.node())
            .expect("node to descend from must exist");
        if handle.idx() < node.len() {
            Some(handle)
        } else {
            None
        }
    }

    /// Removes the key/value pair storage index pointed to by `handle`.
    ///
    /// If through this removal an underfull node was created, appropriate strategies
    /// will be deployed (`.handle_underfull_node`).
    ///
    /// Returns the removed value.
    fn remove_kv(&mut self, handle: KVHandle) -> V {
        debug_assert!(self.header.len > 0);
        self.header.len -= 1;

        let handle_type = self.get_handle_type(handle.node());
        let (small_leaf, old_pair_index, mut new_len) = match handle_type {
            Leaf => self.extract_handle(handle),
            Branch => {
                let child = self
                    .right_child(handle)
                    .expect("every branch node has children; qed");
                let first_leaf = self.first_leaf_edge(child);

                let to_remove = self.right_kv(first_leaf).expect("right_kv must exist");
                let (hole, pair_storage_index, nl) = self.extract_handle(to_remove);

                let node = self.get_node_mut(handle.node()).expect("node must exist");
                let old_pair_index = node
                    .set_pair(handle.idx(), Some(pair_storage_index))
                    .expect("handle must be valid");
                (hole, old_pair_index, nl)
            }
        };

        let mut handle = small_leaf.node();
        while new_len < CAPACITY / 2 {
            match self.handle_underfull_node(handle) {
                UnderflowResult::AtRoot => break,
                UnderflowResult::EmptyParent(_) => unreachable!(
                    "[ink_core::BTreeMap::remove_kv] Error: \
                     while handling an underfull node it was found that its parent \
                     node is empty. This can never happen since we always re-balance the tree \
                     when an element is removed."
                ),
                UnderflowResult::Merged(parent) => {
                    let parent_node =
                        self.get_node(parent).expect("parent node must exist");
                    if parent_node.len() == 0 {
                        self.root_pop_level();
                        break
                    } else {
                        handle = parent;
                        new_len = parent_node.len();
                    }
                }
                UnderflowResult::Stole(_) => break,
            }
        }

        let old = self.remove_pair(old_pair_index).value();
        if new_len == 0 {
            debug_assert_eq!(
                self.get_node(handle)
                    .expect("node must exist")
                    .edges_count(),
                0
            );
            self.remove_node(handle);
            self.header.root = None;
            self.header.next_vacant = None;
            self.header.next_vacant_pair = None;
        }
        old
    }

    /// An underfull node contains less than `CAPACITY / 2` elements and provides
    /// an opportunity to reduce storage space by merging nodes together.
    ///
    /// If merging is not possible we "steal" an element from one node and
    /// put it into its neighboring node.
    ///
    /// The returned `UnderflowResult` contains the result of handling the
    /// underfull node.
    fn handle_underfull_node(&mut self, node: NodeHandle) -> UnderflowResult {
        let parent = if let Some(parent) = self.ascend(node) {
            parent
        } else {
            return UnderflowResult::AtRoot
        };

        let (is_left, handle) = match self.left_kv(parent) {
            Some(left) => (true, left),
            None => {
                match self.right_kv(parent) {
                    Some(right) => (false, right),
                    None => return UnderflowResult::EmptyParent(parent.node()),
                }
            }
        };

        if self.can_merge(handle) {
            self.merge(handle);
            UnderflowResult::Merged(handle.node())
        } else {
            if is_left {
                self.steal_left(handle);
            } else {
                self.steal_right(handle);
            }
            UnderflowResult::Stole(handle.node())
        }
    }

    /// Returns `true` if it is valid to call `.merge()`, i.e. whether there is
    /// enough room in a node to hold the combination of the nodes to the left
    /// and right of `handle` along with an additional key/value pair storage index.
    fn can_merge(&self, handle: KVHandle) -> bool {
        let len_left = self
            .left_child_node(handle)
            .expect("left child must exist")
            .len();
        let len_right = self
            .right_child_node(handle)
            .expect("right child must exist")
            .len();

        len_left + len_right < CAPACITY
    }

    /// Merges the right child of `handle` with the pair storage index pointed
    /// to by `handle` into the left child. The right child is removed.
    ///
    /// Assumes that this edge `.can_merge()`.
    fn merge(&mut self, handle: KVHandle) {
        let right_child = self
            .right_child(handle)
            .expect("right child must exist 591");
        let right_node = self.get_node(right_child).expect("right child must exist");
        let right_edges = right_node.edges;
        let right_pairs = right_node.pairs;
        let right_len = right_node.len();

        let (removed_pair_storage_index, old_node_len) =
            self.extract_handle_for_merge(handle);

        let left_child = self.left_child(handle).expect("left child must exist");
        let left_node = self
            .get_node_mut(left_child)
            .expect("left child must exist");
        let left_len = left_node.len();

        debug_assert!(left_len + right_len < CAPACITY);

        left_node.set_pair(left_len, removed_pair_storage_index);
        let src = &right_pairs[..right_len];
        let start = left_len + 1;
        let end = left_len + 1 + right_len;
        left_node.set_pair_range(src, start..end);
        left_node.set_len(left_len + right_len + 1);

        for i in handle.idx() + 1..old_node_len {
            let h = KVHandle::new(handle.node(), i);
            self.correct_parent_link(h);
        }

        // If the right child has children we need to take care of those
        // by merging the edges of the child into the left node as well.
        if self.has_children(right_child) {
            let from = left_len + 1;
            let to = right_len + 1;
            let left_node = self
                .get_node_mut(left_child)
                .expect("left child must exist");
            left_node.set_edges_range(&right_edges[..to], from..from + to);

            for i in left_len + 1..left_len + right_len + 2 {
                let h = KVHandle::new(left_child, i);
                self.correct_parent_link(h);
            }
        }

        self.remove_node(right_child);
    }

    /// Extracts the storage index referenced by `handle`, decreases the node
    /// length and returns `(removed_pair_storage_index, old_length)`.
    fn extract_handle_for_merge(
        &mut self,
        handle: KVHandle,
    ) -> (Option<KVStorageIndex>, usize) {
        let node = self.get_node_mut(handle.node()).expect("node must exist");
        let _removed_edge = node.remove_edge_with_shift(handle.idx() + 1);
        let removed_pair_storage_index = node.remove_pair_with_shift(handle.idx());
        let old_len = node.len();
        node.set_len(old_len - 1);
        (removed_pair_storage_index, old_len)
    }

    /// Removes a key/value pair storage index from the end of the node referenced by `handle`.
    /// If this is a branch node the edge that was to the right of that pair is also removed.
    ///
    /// Returns `(removed_pair_storage_index, Option<removed_edge>)`.
    fn pop(&mut self, handle: NodeHandle) -> (KVStorageIndex, Option<NodeHandle>) {
        let handle_type = self.get_handle_type(handle);
        let (pair_storage_index, idx) = {
            let node = self.get_node_mut(handle).expect("node must exist");
            debug_assert!(node.len() > 0);
            let idx = node.len() - 1;
            let pair_index = node.take_pair(idx).expect("pair must exist");
            node.set_len(node.len() - 1);
            (pair_index, idx)
        };
        let edge = match handle_type {
            Leaf => None,
            Branch => {
                // If `handle` is a reference to a branch node we also remove the edge right
                // of it.
                let edge = {
                    let node = self.get_node_mut(handle).expect("node must exist");
                    node.take_edge(idx + 1).expect("edge must exist")
                };
                let edge_handle = NodeHandle::new(edge.node());
                self.set_parent(edge_handle, None, None);
                Some(edge_handle)
            }
        };

        (pair_storage_index, edge)
    }

    /// This removes a pair storage index from the left child and replaces it with the
    /// storage index pointed to by `handle` while pushing the old storage index
    /// of `handle` into the right child.
    fn steal_left(&mut self, handle: KVHandle) {
        let left_child = self.left_child(handle).expect("left child must exist");
        let (pair_index, edge) = self.pop(left_child);

        let node = self.get_node_mut(handle.node()).expect("node must exist");
        let pair_storage_index = node
            .set_pair(handle.idx(), Some(pair_index))
            .expect("pair must exist");

        let right = self.right_edge(handle);
        let child = self.descend(right).expect("child must exist");
        match self.get_handle_type(child) {
            Leaf => self.push_front_leaf(child, pair_storage_index),
            Branch => {
                self.push_front_branch(
                    child,
                    pair_storage_index,
                    edge.expect("edge always exists for branch nodes"),
                )
            }
        }
    }

    /// This removes a key/value pair storage index from the right child and replaces it
    /// with the key/value pair index pointed to by `handle` while pushing the old key/value
    /// pair index of `handle` into the left child.
    fn steal_right(&mut self, handle: KVHandle) {
        let right = self.right_edge(handle);
        let child = self.descend(right).expect("child must exist");
        let (k, edge) = self.pop_front(child);

        let node = self.get_node_mut(handle.node()).expect("node must exist");
        let pair_storage_index = node
            .set_pair(handle.idx(), Some(k))
            .expect("pair must exist");

        let left_child = self.left_child(handle).expect("left child must exist");
        match self.get_handle_type(left_child) {
            Leaf => self.push_leaf(left_child, pair_storage_index),
            Branch => {
                self.push_branch(
                    left_child,
                    pair_storage_index,
                    edge.expect("edge always exists for branch node"),
                )
            }
        }
    }

    /// Removes the storage index of a key/value pair from the beginning of this node.
    /// If this is a branch node, also removes the edge that was to the left of that pair.
    ///
    /// *Note:* This method does not actually remove the storage entry! Just the pointer
    /// to the index of it!
    ///
    /// Returns `(removed_storage_index, Option<removed_edge>)`.
    fn pop_front(&mut self, handle: NodeHandle) -> (KVStorageIndex, Option<NodeHandle>) {
        let typ = self.get_handle_type(handle);
        let node = self.get_node_mut(handle).expect("node must exist");

        debug_assert!(node.len() > 0);
        let old_len = node.len();

        let pair_storage_index = node.remove_pair_with_shift(0).expect("pair must exist");

        let edge = match typ {
            Leaf => None,
            Branch => {
                let edge = node.remove_edge_with_shift(0).expect("edge must exist");

                // Create a new `NodeHandle` to the new root
                let new_root = NodeHandle::new(edge.node());
                self.set_parent(new_root, None, None);

                for i in 0..old_len {
                    let h = KVHandle::new(handle, i);
                    self.correct_parent_link(h);
                }

                Some(new_root)
            }
        };

        let node = self.get_node_mut(handle).expect("node must exist");
        node.set_len(node.len() - 1);

        (pair_storage_index, edge)
    }

    /// Adds a key/value pair storage index to the beginning of the node.
    fn push_front_leaf(
        &mut self,
        handle: NodeHandle,
        pair_storage_index: KVStorageIndex,
    ) {
        let node = self.get_node_mut(handle).expect("node must exist");
        debug_assert!(node.len() < CAPACITY);

        node.push_pair_to(0, pair_storage_index);
        node.set_len(node.len() + 1);
    }

    /// Adds a key/value pair storage index to the beginning of the node.
    /// Furthermore an `edge` to go to the left of that pair is added to
    /// the beginning of the node.
    fn push_front_branch(
        &mut self,
        handle: NodeHandle,
        pair_storage_index: KVStorageIndex,
        edge: NodeHandle,
    ) {
        let node = self.get_node_mut(handle).expect("node must exist");
        debug_assert!(node.len() < CAPACITY);

        node.push_pair_to(0, pair_storage_index);
        node.push_edge_to(0, edge);
        node.set_len(node.len() + 1);

        self.correct_all_children_parent_links(handle);
    }

    /// Removes a node by replacing its storage entity with the index of the current
    /// top element in the linked list of vacant storage entries and sets
    /// `header.next_vacant` to the new top element -- `handle`.
    fn remove_node(&mut self, handle: NodeHandle) {
        let n = handle.node();
        let _ = match self.nodes.get(n) {
            None | Some(InternalEntry::Vacant(_)) => None,
            Some(InternalEntry::Occupied(_)) => {
                match self
                    .nodes
                    .put(n, InternalEntry::Vacant(self.header.next_vacant))
                    .expect(
                        "[ink_core::BTreeMap::remove_node] Error: \
                         we already asserted that the entry at `n` exists",
                    ) {
                    InternalEntry::Occupied(val) => {
                        // When removing a node set `next_vacant` to this node index
                        self.header.next_vacant = Some(NodeHandle::new(n));
                        self.header.node_count -= 1;
                        Some(val)
                    }
                    InternalEntry::Vacant(_) => {
                        unreachable!(
                            "[ink_core::BTreeMap::remove_node] Error: \
                             we already asserted that the entry is occupied"
                        )
                    }
                }
            }
        };
    }

    /// Removes a pair by replacing its storage entity with the index of the current
    /// top element in the linked list of vacant storage entries and setting
    /// `header.next_vacant_pair` to the new top element -- `handle`.
    fn remove_pair(&mut self, storage_index: KVStorageIndex) -> KVPair<K, V> {
        let pair = match self.kv_pairs.get(storage_index) {
            None | Some(InternalKVEntry::Vacant(_)) => None,
            Some(InternalKVEntry::Occupied(_)) => {
                match self
                    .kv_pairs
                    .put(
                        storage_index,
                        InternalKVEntry::Vacant(self.header.next_vacant_pair),
                    )
                    .expect(
                        "[ink_core::BTreeMap::remove_pair] Error: \
                         we already asserted that the pair at `n` exists",
                    ) {
                    InternalKVEntry::Occupied(val) => {
                        // When removing a pair set `next_vacant_pair` to this node index
                        self.header.next_vacant_pair = Some(storage_index);
                        Some(val)
                    }
                    InternalKVEntry::Vacant(_) => {
                        unreachable!(
                            "[ink_core::BTreeMap::remove_pair] Error: \
                             we already asserted that the pair is occupied"
                        )
                    }
                }
            }
        };
        pair.expect("must exist")
    }

    /// Extracts the key/value pair storage index pointed to by `handle`, returning
    /// the edge between the now adjacent key/value pairs to the left and right of `handle`.
    ///
    /// Returns `(left_edge, removed_pair_storage_index, old_val, new_node_len)`.
    fn extract_handle(&mut self, handle: KVHandle) -> (KVHandle, KVStorageIndex, usize) {
        let node = self.get_node_mut(handle.node()).expect("node must exist");
        let pair_storage_index = node
            .remove_pair_with_shift(handle.idx())
            .expect("pair must exist");
        node.set_len(node.len() - 1);
        let new_len = node.len();
        (self.left_edge(handle), pair_storage_index, new_len)
    }

    /// Returns a mutable reference to the node referenced by `handle`.
    fn get_node_mut(&mut self, handle: NodeHandle) -> Option<&mut Node<K, V>> {
        let entry = self.nodes.get_mut(handle.node())?;
        match entry {
            InternalEntry::Occupied(occupied) => Some(occupied),
            InternalEntry::Vacant(_) => None,
        }
    }

    /// Put the element into the tree at the next vacant position.
    ///
    /// Returns the tree index that the element was put into.
    fn put(&mut self, node: Node<K, V>) -> NodeHandle {
        let node_handle = match self.header.next_vacant {
            None => {
                // then there is no vacant entry which we can reuse
                self.nodes
                    .set(self.node_count(), InternalEntry::Occupied(node));
                NodeHandle::new(self.node_count())
            }
            Some(current_vacant) => {
                // then there is a vacant entry which we can reuse
                let next_vacant = match self
                    .nodes
                    .put(current_vacant.node(), InternalEntry::Occupied(node))
                    .expect(
                        "[ink_core::BTreeMap::put] Error: \
                         expected a vacant entry here, but no entry was found",
                    ) {
                    InternalEntry::Vacant(next_vacant) => next_vacant,
                    InternalEntry::Occupied(_) => {
                        unreachable!(
                            "[ink_core::BTreeMap::put] Error: \
                             a next_vacant index can never point to an occupied entry"
                        )
                    }
                };
                // when putting node set next_vacant to the next_vacant which was found here
                self.header.next_vacant = next_vacant;
                current_vacant
            }
        };
        self.header.node_count += 1;
        node_handle
    }

    /// Put a key/value pair storage index into the tree at the next vacant position.
    ///
    /// Returns the storage index that the element was put into.
    fn put_pair(&mut self, pair: KVPair<K, V>) -> KVStorageIndex {
        match self.header.next_vacant_pair {
            None => {
                // then there is no vacant entry which we can reuse
                self.kv_pairs
                    .set(self.len(), InternalKVEntry::Occupied(pair));
                self.len()
            }
            Some(current_vacant) => {
                // then there is a vacant entry which we can reuse
                let next_vacant = match self
                    .kv_pairs
                    .put(current_vacant, InternalKVEntry::Occupied(pair))
                    .expect(
                        "[ink_core::BTreeMap::put_pair] Error: \
                         expected a vacant entry here, but no entry was found",
                    ) {
                    InternalKVEntry::Vacant(next_vacant) => next_vacant,
                    InternalKVEntry::Occupied(_) => {
                        unreachable!(
                            "[ink_core::BTreeMap::put_pair] Error: \
                             a next_vacant index can never point to an occupied entry"
                        )
                    }
                };
                // when putting node set next_vacant to the next_vacant which was found here
                self.header.next_vacant_pair = next_vacant;
                current_vacant
            }
        }
    }

    /// Adds the storage index of a key/value pair to the end of the `dst` node.
    /// Furthermore an `edge` is added right of that pair, at the end of the node.
    fn push_branch(
        &mut self,
        dst: NodeHandle,
        pair_storage_index: KVStorageIndex,
        edge: NodeHandle,
    ) {
        let node = self.get_node_mut(dst).expect("destination node must exist");
        node.set_pair(node.len(), Some(pair_storage_index));
        node.set_edge(node.len() + 1, Some(edge));

        let handle = KVHandle::new(dst, node.len() + 1);
        node.set_len(node.len() + 1);
        self.correct_parent_link(handle);
    }

    /// Adds the storage index of a key/value pair to the end of the `dst` node.
    fn push_leaf(&mut self, dst: NodeHandle, pair_storage_index: KVStorageIndex) {
        let node = self.get_node_mut(dst).expect("destination node must exist");

        debug_assert!(node.len() < CAPACITY);

        let idx = node.len();
        node.set_pair(idx, Some(pair_storage_index));
        node.set_len(node.len() + 1);
    }

    /// Splits the underlying node into three parts:
    ///
    /// - The node is truncated to only contain the key/value pair storage index
    ///   to the right of `handle`.
    /// - The key and value pointed to by `handle` and extracted.
    /// - All the key/value pairs to the right of `handle` are put into a newly
    ///   allocated node.
    ///
    /// Returns a tuple of `(extracted_pair_storage_index, handle_to_new_node)`.
    fn split_leaf(
        &mut self,
        handle: NodeHandle,
        idx: usize,
    ) -> (KVStorageIndex, NodeHandle) {
        let node = self.get_node_mut(handle).expect("node to split must exist");

        // We can only start splitting at leaf nodes.
        debug_assert_eq!(node.edges_count(), 0);

        let mut right = Node::new();
        let pair_storage_index = node
            .take_pair(idx)
            .expect("pair must exist at split location");
        node.set_len(node.len() - 1);

        let from = idx + 1;
        for i in from..CAPACITY {
            let a = i - from;
            right.set_pair(a, node.take_pair(i));
            if right.pair(a).is_some() {
                if node.len() > 0 {
                    node.set_len(node.len() - 1);
                }
                right.set_len(right.len() + 1);
            }
        }

        let right_handle = self.put(right);
        (pair_storage_index, right_handle)
    }

    /// Splits the underlying node into three parts:
    ///
    /// - The node is truncated to only contain the edges and key/value pair index
    ///   to the right of `handle`.
    /// - The key and value pointed to by `handle` and extracted.
    /// - All the edges and key/value pair storage index to the right of `handle` are
    ///   put into a newly allocated node.
    ///
    /// Returns a tuple of `(extracted_key, extracted_value, handle_to_new_node)`.
    fn split_branch(
        &mut self,
        parent: NodeHandle,
        idx: usize,
    ) -> (KVStorageIndex, NodeHandle) {
        let node = self.get_node_mut(parent).expect("node to split must exist");

        let count = node.len();
        let new_len = count - idx - 1;

        let mut right = Node::new();
        right.set_parent(Some(parent));
        right.set_parent_idx(Some(idx));

        let pair_storage_index = node
            .take_pair(idx)
            .expect("pair must exist at split location");
        node.set_len(node.len() - 1);

        let from = idx + 1;
        for a in 0..new_len {
            let i = from + a;
            right.set_pair(a, node.take_pair(i));
            if right.pair(a).is_some() {
                node.set_len(node.len() - 1);
                right.set_len(right.len() + 1);
            }
        }
        for a in 0..=new_len {
            let i = from + a;
            right.set_edge(a, node.take_edge(i));
        }

        let right_handle = self.put(right);
        for i in 0..=new_len {
            let handle = KVHandle::new(right_handle, i);
            self.correct_parent_link(handle);
        }

        (pair_storage_index, right_handle)
    }

    /// Adds a new branch node with a single edge, pointing to the previous root, and make that
    /// new node the root. This increases the height by 1 and is the opposite of `pop_level`.
    ///
    /// Returns a handle to the new root node.
    fn root_push_level(&mut self) -> NodeHandle {
        let current_root_handle = self.header.root.expect("node must exist");

        let mut new_root = Node::<K, V>::new();
        new_root.set_edge(0, Some(current_root_handle));
        let new_root_handle = self.put(new_root);

        self.set_parent(
            self.header.root.expect("root must exist"),
            Some(new_root_handle),
            Some(0),
        );

        self.header.root = Some(new_root_handle);
        new_root_handle
    }

    /// Removes the root node, using its first child as the new root. This cannot be called when
    /// the tree consists only of a leaf node. As it is intended only to be called when the root
    /// has only one edge, no cleanup is done on any of the other children are elements of the root.
    /// This decreases the height by 1 and is the opposite of `push_level`.
    fn root_pop_level(&mut self) {
        let handle = self.header.root.expect("root must exist");
        debug_assert!(
            self.get_node(handle)
                .expect("root must exist")
                .edges_count()
                == 1
        );
        let edge = self.first_edge(handle);

        let child = self.descend(edge).expect("child must exist");
        self.set_parent(child, None, None);

        self.header.root = Some(child);

        self.remove_node(handle);
    }

    /// Sets the parent node of `handle` to `parent` and its `idx` to `parent_idx`.
    fn set_parent(
        &mut self,
        handle: NodeHandle,
        parent_node: Option<NodeHandle>,
        parent_idx: Option<usize>,
    ) {
        let node = self.get_node_mut(handle).expect("node must exist");
        node.set_parent(parent_node);
        node.set_parent_idx(parent_idx);
    }

    /// Inserts a key/value pair storage index at `handle`. If this results in an overfull
    /// node the node is split.
    ///
    /// Returns the `(result_of_insert_operation, mutable_reference_to_inserted_value)`.
    fn insert_into_node(
        &mut self,
        handle: KVHandle,
        pair_storage_index: KVStorageIndex,
    ) -> (InsertResult, KVStorageIndex) {
        let node = self
            .get_node(handle.node())
            .expect("node to insert into must exist");
        let len = node.len();

        let pair = self
            .get_kv_ref(pair_storage_index)
            .expect("requested pair must always exist");
        let k = &pair.key_ref();

        if len < CAPACITY {
            let h = match search::search_node(
                node,
                self.keys_in_node(handle.node()),
                handle.node(),
                k,
            ) {
                Found(h) => h,
                NotFound(h) => h,
            };
            let a = self.insert_fit(h, pair_storage_index);
            self.header.len += 1;
            (Fit(handle), a)
        } else {
            let (extracted_pair_index, right) = self.split_leaf(handle.node(), B);

            let index = if handle.idx() <= B {
                // handle is left side
                self.insert_fit(handle, pair_storage_index)
            } else {
                let h = KVHandle::new(right, handle.idx() - (B + 1));
                self.insert_fit(h, pair_storage_index)
            };

            (Split(handle, extracted_pair_index, right), index)
        }
    }

    /// Insert `K`, `V` and `edge` into `handle` if it fits. If it does not fit the node
    /// referenced by `handle` is split.
    ///
    /// Returns the result of the insert operation.
    fn insert_into_branch_node(
        &mut self,
        handle: KVHandle,
        pair_storage_index: KVStorageIndex,
        edge: NodeHandle,
    ) -> InsertResult {
        let pair = self
            .get_kv_ref(pair_storage_index)
            .expect("requested pair must always exist");
        let k = &pair.key_ref();

        let node = self
            .get_node(handle.node())
            .expect("parent to insert into must exist");

        if node.len() < CAPACITY {
            let kv_handle = match search::search_node(
                node,
                self.keys_in_node(handle.node()),
                handle.node(),
                &k,
            ) {
                Found(h) => h,
                NotFound(h) => h,
            };
            self.insert_fit_edge(kv_handle, pair_storage_index, edge);
            Fit(kv_handle)
        } else {
            let (extracted_pair_index, right) = self.split_branch(handle.node(), B);
            if handle.idx() <= B {
                // Handle is left side.
                self.insert_fit_edge(handle, pair_storage_index, edge);
            } else {
                let h = KVHandle::new(right, handle.idx() - (B + 1));
                self.insert_fit_edge(h, pair_storage_index, edge);
            }

            Split(handle, extracted_pair_index, right)
        }
    }

    /// Inserts a new key/value pair storage index between the key/value pairs to the right
    /// and left of this edge. This method assumes that there is enough space in the node for
    /// the new pair to fit.
    ///
    /// The returned storage index points to the inserted value.
    fn insert_fit(
        &mut self,
        handle: KVHandle,
        pair_storage_index: KVStorageIndex,
    ) -> KVStorageIndex {
        let node = self
            .get_node_mut(handle.node())
            .expect("node to insert_fit into must exist");
        debug_assert!(node.len() < CAPACITY);

        node.push_pair_to(handle.idx(), pair_storage_index);
        node.set_len(node.len() + 1);
        pair_storage_index
    }

    /// Inserts a new key/value pair storage index and an edge that will go to the right of
    /// that new pair between this edge and the key/value pair storage index to the right
    /// of this edge.
    /// This method assumes that there is enough space in the node for the new pair to fit.
    fn insert_fit_edge(
        &mut self,
        handle: KVHandle,
        pair_storage_index: KVStorageIndex,
        edge: NodeHandle,
    ) {
        self.insert_fit(handle, pair_storage_index);

        let node = self
            .get_node_mut(handle.node())
            .expect("node to insert (k, v, edge) into must exist");

        node.push_edge_to(handle.idx() + 1, edge);

        for idx in (handle.idx() + 1)..=node.len() {
            let handle = KVHandle::new(handle.node(), idx);
            self.correct_parent_link(handle);
        }
    }

    /// Returns the node of the left child of `handle`.
    fn left_child_node(&self, handle: KVHandle) -> Option<&Node<K, V>> {
        let child = self.left_child(handle)?;
        self.get_node(child)
    }

    /// Returns the node of the right child of `handle`.
    fn right_child_node(&self, handle: KVHandle) -> Option<&Node<K, V>> {
        let child = self.right_child(handle)?;
        self.get_node(child)
    }

    /// Returns a handle to the left child of `handle`.
    fn left_child(&self, handle: KVHandle) -> Option<NodeHandle> {
        let left = self.left_edge(handle);
        self.descend(left)
    }

    /// Returns a handle to the right child of `handle`.
    fn right_child(&self, handle: KVHandle) -> Option<NodeHandle> {
        let right = self.right_edge(handle);
        self.descend(right)
    }

    /// Fixes the parent pointer and index in the child node below this edge. This is useful
    /// when the ordering of edges has been changed, such as in the various `insert` methods.
    /// Then the parent pointer in the child nodes has to be updated, otherwise they would
    /// point to some entry in `edges` which is no longer correct.
    fn correct_parent_link(&mut self, handle: KVHandle) {
        let child = self
            .descend(handle)
            .expect("child in which to correct parent link must exist");
        self.set_parent(child, Some(handle.node()), Some(handle.idx()));
    }

    /// Iterates through all children of a node and fixes the parent pointer in the child.
    /// Each child contains a pointer to a position in the `edges` array. Sometimes (e.g. for
    /// `insert` operations) the order in the `edges` array changes and the child pointers
    /// have to be corrected.
    fn correct_all_children_parent_links(&mut self, handle: NodeHandle) {
        let node = self.get_node(handle).expect("node must exist");
        let len = node.len();

        for i in 0..=len {
            let h = KVHandle::new(handle, i);
            self.correct_parent_link(h);
        }
    }

    /// Returns `true` if a node has children.
    fn has_children(&mut self, handle: NodeHandle) -> bool {
        let node = self.get_node(handle).expect("node must exist");
        node.edges_count() > 0
    }
}

impl<K, V> BTreeMap<K, V>
where
    K: Eq + Ord + Codec,
    V: Codec,
{
    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_compile
    /// use ink_core::storage::BTreeMap;
    ///
    /// let mut map = new_btree_map();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        match search::search_tree(&self, key) {
            Found(handle) => {
                let v = self
                    .get_value(handle)
                    .expect("if found there is always a value; qed");
                Some(v)
            }
            NotFound(_) => None,
        }
    }

    /// Returns the key/value pair corresponding to `key`.
    ///
    /// The supplied key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// # Examples
    ///
    /// ```no_compile
    /// use ink_core::collections::BTreeMap;
    ///
    /// let mut map = new_btree_map();
    /// map.insert(1, "a");
    /// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
    /// assert_eq!(map.get_key_value(&2), None);
    /// ```
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        match search::search_tree(&self, key) {
            Found(handle) => self.get_kv(handle).map(|kv| kv.kv()),
            NotFound(_) => None,
        }
    }

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_compile
    /// use ink_core::collections::BTreeMap;
    ///
    /// let mut map = new_btree_map();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.get(key).is_some()
    }

    /// Inserts a key/value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical. See the [module-level
    /// documentation] for more.
    ///
    /// [module-level documentation]: index.html#insert-and-complex-keys
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_compile
    /// use ink_core::storage::BTreeMap;
    ///
    /// let mut map = new_btree_map();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map[&37], "c");
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.entry(key) {
            Entry::Occupied(mut entry) => entry.insert(value),
            Entry::Vacant(entry) => {
                entry.insert(value);
                None
            }
        }
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_compile
    /// use ink_core::storage::BTreeMap;
    ///
    /// let mut map = new_btree_map();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(&1), Some("a"));
    /// assert_eq!(map.remove(&1), None);
    /// ```
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        match search::search_tree(&self, key) {
            Found(handle) => {
                let o = OccupiedEntry { tree: self, handle };
                Some(o.remove())
            }
            NotFound(_) => None,
        }
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_compile
    /// use ink_core::storage::BTreeMap;
    ///
    /// let mut count: BTreeMap<&str, usize> = new_btree_map();
    ///
    /// // count the number of occurrences of letters in the vec
    /// for x in vec!["a","b","a","c","a","b"] {
    ///     *count.entry(x).or_insert(0) += 1;
    /// }
    ///
    /// assert_eq!(count["a"], 3);
    /// ```
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        match search::search_tree(&self, &key) {
            Found(handle) => Entry::Occupied(OccupiedEntry { tree: self, handle }),
            NotFound(handle) => {
                Entry::Vacant(VacantEntry {
                    key: Some(key),
                    tree: self,
                    handle,
                })
            }
        }
    }
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Ord + Codec,
    V: Codec,
{
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```no_compile
    /// use ink_core::storage::BTreeMap;
    ///
    /// let mut map: BTreeMap<&str, usize> = new_btree_map();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// ```
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Returns a reference to this entry's key.
    ///
    /// # Examples
    ///
    /// ```no_compile
    /// use ink_core::storage::BTreeMap;
    ///
    /// let mut map: BTreeMap<&str, usize> = new_btree_map();
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    pub fn key(&self) -> &K {
        match *self {
            Entry::Occupied(ref entry) => entry.key(),
            Entry::Vacant(ref entry) => entry.key(),
        }
    }
}

#[cfg(feature = "ink-generate-abi")]
impl<K, V> HasLayout for BTreeMap<K, V>
where
    K: Metadata + 'static,
    V: Metadata + 'static,
{
    fn layout(&self) -> StorageLayout {
        LayoutStruct::new(
            Self::meta_type(),
            vec![
                LayoutField::of("header", &self.header),
                LayoutField::of("entries", &self.nodes),
                LayoutField::of("kv_pairs", &self.kv_pairs),
            ],
        )
        .into()
    }
}

/// The result of an insert operation.
enum InsertResult {
    /// The element did fit into the node.
    Fit(KVHandle),
    /// The element didn't fit into the node and the node was split.
    /// The element behind `KVStorageIndex` is extracted during this split
    /// and now needs to be inserted into a new place.
    /// `KVHandle` references the resulting left node, `NodeHandle`
    /// the right one.
    Split(KVHandle, KVStorageIndex, NodeHandle),
}

/// A storage entity which contains either an occupied entry with a tree node
/// or a vacant entry pointing to the next vacant entry.
///
/// Using this mechanism we build a linked list of vacant storage entries. On each
/// insert we replace the top entry (`header.next_vacant`) of this vacant list with
/// an `OccupiedEntry` and set `header.next_vacant` to the next element in the list.
///
/// In our implementation we distinguish between `InternalEntry` and `Entry`.
///   - `Entry` is the public facing enum which is used in conjunction with the
///     `.entry()` API. It contains a key/value pair.
///   - `InternalEntry` is used internally in our implementation. It is a storage
///     entity and contains a tree node with many key/value pair storage indices.
#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
enum InternalEntry<K, V> {
    /// A vacant entry pointing to the next vacant index.
    Vacant(Option<NodeHandle>),
    /// An occupied entry contains a tree node with its elements.
    Occupied(Node<K, V>),
}

impl<K, V> Flush for InternalEntry<K, V>
where
    K: Encode + Flush,
    V: Encode + Flush,
{
    #[inline]
    fn flush(&mut self) {
        match self {
            InternalEntry::Vacant(vacant) => vacant.flush(),
            InternalEntry::Occupied(occupied) => occupied.flush(),
        }
    }
}

/// A storage entity which contains either an occupied entry with a tree node
/// or a vacant entry pointing to the next vacant entry.
///
/// Using this mechanism we build a linked list of vacant storage entries. On each
/// insert we replace the top entry (`header.next_vacant`) of this vacant list with
/// an `OccupiedEntry` and set `header.next_vacant` to the next element in the list.
///
/// In our implementation we distinguish between `InternalEntry` and `Entry`.
///   - `Entry` is the public facing enum which is used in conjunction with the
///     `.entry()` API. It contains a key/value pair.
///   - `InternalEntry` is used internally in our implementation. It is a storage
///     entity and contains a tree node with many key/value pairs.
#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
enum InternalKVEntry<K, V> {
    /// A vacant entry pointing to the next vacant index.
    Vacant(Option<KVStorageIndex>),
    /// An occupied entry contains a key/value pair.
    Occupied(KVPair<K, V>),
}

impl<K, V> Flush for InternalKVEntry<K, V>
where
    K: Encode + Flush,
    V: Encode + Flush,
{
    #[inline]
    fn flush(&mut self) {
        match self {
            InternalKVEntry::Vacant(vacant) => vacant.flush(),
            InternalKVEntry::Occupied(occupied) => occupied.flush(),
        }
    }
}

/// An entry of a storage map.
///
/// This can either store the entries key/value pair or represent an entry that was
/// removed after it has been occupied with key and value.
#[derive(Encode, Decode)]
pub enum Entry<'a, K, V> {
    /// A vacant entry pointing to the next vacant index.
    Vacant(VacantEntry<'a, K, V>),
    /// An occupied entry containing the value.
    Occupied(OccupiedEntry<'a, K, V>),
}

/// An occupied entry of a storage map.
pub struct OccupiedEntry<'a, K, V> {
    tree: &'a mut BTreeMap<K, V>,
    handle: KVHandle,
}

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: Encode + Decode + Ord,
    V: Encode + Decode,
{
    /// Gets a reference to the key that would be used when inserting a value
    /// through the VacantEntry.
    ///
    /// # Examples
    ///
    /// ```no_compile
    /// use ink_core::storage::BTreeMap;
    ///
    /// let mut map: BTreeMap<&str, usize> = new_btree_map();
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    pub fn key(&self) -> &K {
        self.key
            .as_ref()
            .expect("entry does always have a key; qed")
    }

    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns a mutable reference to it.
    ///
    /// # Examples
    ///
    /// ```no_compile
    /// use ink_core::storage::BTreeMap;
    ///
    /// let mut count: BTreeMap<&str, usize> = new_btree_map();
    ///
    /// // count the number of occurrences of letters in the vec
    /// for x in vec!["a","b","a","c","a","b"] {
    ///     *count.entry(x).or_insert(0) += 1;
    /// }
    ///
    /// assert_eq!(count["a"], 3);
    /// ```
    pub fn insert(mut self, val: V) -> &'a mut V {
        let key = self
            .key
            .take()
            .expect("key is only taken when inserting, so must be there; qed");

        let pair_index = if self.tree.is_empty() && self.tree.root().is_none() {
            self.tree.create_root(key, val)
        } else {
            self.tree.insert_kv(self.handle, key, val)
        };
        self.tree
            .get_value_mut_ref(pair_index)
            .expect("value was just inserted; qed")
    }
}

impl<'a, K, V> OccupiedEntry<'a, K, V>
where
    K: Encode + Decode + Ord,
    V: Encode + Decode,
{
    /// Gets a reference to the key in the entry.
    ///
    /// # Examples
    ///
    /// ```no_compile
    /// use ink_core::storage::BTreeMap;
    ///
    /// let mut map: BTreeMap<&str, usize> = new_btree_map();
    /// map.entry("poneyland").or_insert(12);
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    pub fn key(&self) -> &K {
        self.tree
            .get_kv(self.handle)
            .expect("every occupied entry always has a key/value pair; qed")
            .key()
    }

    /// Gets a reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```no_compile
    /// use ink_core::storage::BTreeMap;
    /// use ink_core::storage::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = new_btree_map();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     assert_eq!(o.get(), &12);
    /// }
    /// ```
    pub fn get(&self) -> &V {
        self.tree
            .get_value(self.handle)
            .expect("every occupied entry always has a key/value pair; qed")
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```no_compile
    /// use ink_core::storage::BTreeMap;
    /// use ink_core::storage::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = new_btree_map();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// if let Entry::Occupied(mut o) = map.entry("poneyland") {
    ///     *o.get_mut() += 10;
    ///     assert_eq!(*o.get(), 22);
    ///
    ///     // We can use the same Entry multiple times.
    ///     *o.get_mut() += 2;
    /// }
    /// assert_eq!(map["poneyland"], 24);
    /// ```
    pub fn get_mut(&mut self) -> &mut V {
        let idx = self.handle.idx();
        let node = self
            .tree
            .get_node_mut(self.handle.node())
            .expect("every occupied entry always belongs to a node; qed");
        let storage_index = node
            .pair(idx)
            .expect("every occupied entry always has a pair stored in it; qed");
        self.tree
            .get_value_mut_ref(storage_index)
            .expect("every pair always has a value; qed")
    }

    /// Converts the entry into a mutable reference to its value.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    /// [`get_mut`]: #method.get_mut
    ///
    /// # Examples
    ///
    /// ```no_compile
    /// use ink_core::storage::BTreeMap;
    /// use ink_core::storage::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = new_btree_map();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     *o.into_mut() += 10;
    /// }
    /// assert_eq!(map["poneyland"], 22);
    /// ```
    pub fn into_mut(self) -> &'a mut V {
        self.into_value_mut()
            .expect("every occupied entry always has a key/value pair; qed")
    }

    /// Takes the value of the entry out of the map, and returns it.
    ///
    /// # Examples
    ///
    /// ```no_compile
    /// use ink_core::storage::BTreeMap;
    /// use ink_core::storage::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = new_btree_map();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     assert_eq!(o.remove(), 12);
    /// }
    /// // If we try to get "poneyland"'s value, it'll panic:
    /// // println!("{}", map["poneyland"]);
    /// ```
    pub fn remove(self) -> V {
        self.tree.remove_kv(self.handle)
    }

    /// Inserts a value into this entry.
    /// Returns the replaced value.
    fn insert(&mut self, value: V) -> Option<V> {
        let node = self
            .tree
            .get_node_mut(self.handle.node())
            .expect("every occupied entry always belongs to a node; qed");

        let storage_index = node
            .pair(self.handle.idx())
            .expect("each occupied entry must already have a pair");
        let entry = self
            .tree
            .kv_pairs
            .take(storage_index)
            .expect("each occupied entry must already have a pair");
        match entry {
            InternalKVEntry::Vacant(_) => {
                unreachable!(
                    "[ink_core::BTreeMap::insert] Error: \
                     we already asserted that the entry is occupied"
                )
            }
            InternalKVEntry::Occupied(occupied) => {
                let (key, old_value) = occupied.kv();
                let new_pair = KVPair::<K, V>::new(key, value);
                self.tree
                    .kv_pairs
                    .set(storage_index, InternalKVEntry::Occupied(new_pair));
                Some(old_value)
            }
        }
    }

    /// Transforms this object into a mutable reference to the value in it.
    fn into_value_mut(self) -> Option<&'a mut V> {
        let storage_index = {
            let idx = self.handle.idx();
            let node = self.tree.get_node_mut(self.handle.node())?;
            let index = node.pair(idx).as_ref()?;
            *index
        };
        self.tree.get_value_mut_ref(storage_index)
    }
}

/// A vacant entry of a storage map.
pub struct VacantEntry<'a, K, V> {
    // The `key` needs to be moved for putting, hence we have to use `Option<K`>
    // to prevent running into partial move errors.
    key: Option<K>,
    /// A reference to the `BTreeMap`.
    tree: &'a mut BTreeMap<K, V>,
    /// A reference to a specific point in the tree.
    handle: KVHandle,
}
