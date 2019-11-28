// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use crate::storage::{
    self,
    alloc::{
        Allocate,
        AllocateUsing,
        Initialize,
    },
    chunk::SyncChunk,
    Flush,
};
#[cfg(feature = "ink-generate-abi")]
use ink_abi::{
    HasLayout,
    LayoutField,
    LayoutStruct,
    StorageLayout,
};
use scale::{
    Decode,
    Encode,
};
#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;
use core::{
    cmp::Ord,
    ptr,
};
use super::search::{self, SearchResult};

/// Each node in the tree has 2 * B children.
pub(crate) const B: usize = 6;

/// Number of elements which can be stored in one node of the tree.
pub const CAPACITY: usize = 2 * B - 1;

/// Mapping stored in the contract storage.
///
/// ToDo add more verbose comments
#[derive(Debug)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct BTreeMap<K, V> {
    /// The entries of the map.
    tree: Tree<K, V>,
}

/// ToDo
#[derive(Debug)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub(crate) struct Tree<K, V> {
    /// Stores densely packed general BTreeMap information.
    header: storage::Value<BTreeMapHeader>,
    /// The entries of the map.
    entries: SyncChunk<InternalEntry<K, V>>
}

impl<K, V> Flush for Tree<K, V>
where
    K: Encode + Flush,
    V: Encode + Flush,
{
    #[inline]
    fn flush(&mut self) {
        self.header.flush();
        self.entries.flush();
    }
}

impl<K, V> Encode for Tree<K, V> {
    fn encode_to<W: scale::Output>(&self, dest: &mut W) {
        self.header.encode_to(dest);
        self.entries.encode_to(dest);
    }
}

impl<K, V> Decode for Tree<K, V> {
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        let header = storage::Value::decode(input)?;
        let entries = SyncChunk::decode(input)?;
        Ok(Self { header, entries })
    }
}

impl<K, V> AllocateUsing for Tree<K, V> {
    #[inline]
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
        where
            A: Allocate,
    {
        Self {
            header: storage::Value::allocate_using(alloc),
            entries: SyncChunk::allocate_using(alloc),
        }
    }
}

impl<K, V> Tree<K, V>
{
    pub fn initialize(&mut self) {
        self.header.set(BTreeMapHeader {
            next_vacant: 0,
            len: 0,
            node_count: 0,
            root: 0,
        });
    }

    /// Returns the number of elements stored in the map.
    pub fn len(&self) -> u32 {
        self.header.len
    }

    /// Returns the number of elements stored in the map.
    pub fn inc_len(&mut self) {
        self.header.len += 1;
    }

    /// Returns the index of the current root.
    pub fn root(&self) -> u32 {
        self.header.root
    }
}

impl<K, V> Tree<K, V>
where
    K: Encode + Decode + Ord + core::fmt::Debug,
    V: Encode + Decode + core::fmt::Debug,
{
    /// Just for debugging purposes.
    /// ToDo remove
    pub fn print_tree(&self) {
        for i in 0..self.header.node_count {
            let nd = NodeHandle(i);
            let node = self.get_node(&nd).expect("printing all nodes");
            eprintln!(
                "{:?}\nkeys: {:?}\nvals: {:?}\nedges: {:?}\nparent: {:?}\nparent idx: {:?}\n",
                i, node.keys, node.vals, node.edges, node.parent, node.parent_idx);
        }
    }

    pub fn insert<'a>(&'a mut self, handle: KVHandle, key: K, val: V) -> &'a mut V {
        if self.len() == 0 {
            let mut node = Node::<K, V>::new();
            node.keys[0] = Some(key);
            node.vals[0] = Some(val);
            node.len = 1;

            let index = self.put(node);
            self.inc_len();

            let node = self.get_node_mut(&index.into())
                .expect("index was just put; qed");
            let val: *mut V = node.vals[0].as_mut()
                .expect("value was just inserted; qed");
            return unsafe { &mut *val };
        }

        let mut ins_k;
        let mut ins_v;
        let mut ins_edge;
        let out_ptr;

        let mut cur_parent = match self.insert_into_node(handle, key, val) {
            (InsertResult::Fit(_), ptr) => {
                return unsafe { &mut *ptr };
            }
            (InsertResult::Split(left, k, v, right), ptr) => {
                ins_k = k;
                ins_v = v;
                ins_edge = right;
                out_ptr = ptr;
                self.ascend(&NodeHandle(left.node))
            }
        };

        loop {
            match cur_parent {
                Some(handle) => {
                    let parent = handle;
                    match self.insert_into_parent(parent, ins_k, ins_v, ins_edge) {
                        InsertResult::Fit(_) => {
                            self.inc_len();
                            return unsafe { &mut *out_ptr };
                        },
                        InsertResult::Split(left, k, v, right) => {
                            ins_k = k;
                            ins_v = v;
                            ins_edge = right;
                            cur_parent = self.ascend(&NodeHandle(left.node));
                        }
                    }
                }
                None => {
                    let new_root = self.root_push_level();
                    self.push(new_root, ins_k, ins_v, ins_edge);
                    return unsafe { &mut *out_ptr };
                }
            }
        }

    }

    /// Finds the node pointed to by this edge.
    ///
    /// `edge.descend().ascend().unwrap()` and `node.ascend().unwrap().descend()` should
    /// both, upon success, do nothing.
    pub fn descend(&self, handle: &KVHandle) -> Option<NodeHandle> {
        let node = self.get_node(&handle.into())
            .expect("node to descend from must exist; qed");
        node.edges[handle.idx()].map(|e| NodeHandle(e))
    }

    /// Finds the node pointed to by this edge.
    ///
    /// `edge.descend().ascend().unwrap()` and `node.ascend().unwrap().descend()` should
    /// both, upon success, do nothing.
    pub fn ascend(&self, handle: &NodeHandle) -> Option<KVHandle> {
        let node = self.get_node(handle)
            .expect("node to ascend from must exist");

        node.parent.map(|parent|
            KVHandle::new(parent, node.parent_idx.expect("parent_idx must exist"))
        )
    }

    /// Fetches a reference to the node behind the supplied handle.
    pub fn get_node(&self, handle: &NodeHandle) -> Option<&Node<K, V>> {
        let entry = self.entries.get(handle.0)
            .expect("node to get must exist; qed");
        match entry {
            InternalEntry::Occupied(occupied) => Some(occupied),
            InternalEntry::Vacant(_) => None,
        }
    }

    /// Fetches a mutable reference to the node behind the supplied handle.
    fn get_node_mut(&mut self, handle: &NodeHandle) -> Option<&mut Node<K, V>> {
        match self.entries.get_mut(handle.0) {
            Some(entry) =>
                match entry {
                    InternalEntry::Occupied(occupied) => Some(occupied),
                    InternalEntry::Vacant(_) => None,
                }
            None => None
        }
    }

    /// Put the element into the tree at the next vacant position.
    ///
    /// Returns the tree index that the element was put into.
    fn put(&mut self, node: Node<K, V>) -> u32 {
        let current_vacant = self.header.next_vacant;

        debug_assert!(current_vacant <= self.len());

        if current_vacant == self.header.node_count {
            self.entries.set(current_vacant, InternalEntry::Occupied(node));
            self.header.next_vacant = current_vacant + 1;
        } else {
            let next_vacant = match self
                .entries
                .put(current_vacant, InternalEntry::Occupied(node))
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
            self.header.next_vacant = next_vacant;
        }
        self.header.node_count += 1;
        current_vacant
    }

    /// Adds a key/value pair and an edge to go to the right of that pair to
    /// the end of the node.
    pub fn push(&mut self, dst: NodeHandle, key: K, val: V, edge: NodeHandle) {
        self.header.len += 1;
        let mut node = self.get_node_mut(&dst)
            .expect("destination node to push to must exist");
        node.keys[node.len()] = Some(key);
        node.vals[node.len()] = Some(val);
        node.edges[node.len() + 1] = Some(edge.0);

        let handle = KVHandle::new(dst.0, node.len() as u32 + 1);
        node.len += 1;
        self.correct_parent_link(handle);
    }

    /// Splits the underlying node into three parts:
    ///
    /// - The node is truncated to only contain the key/value pairs to the right of
    ///   this handle.
    /// - The key and value pointed to by this handle and extracted.
    /// - All the key/value pairs to the right of this handle are put into a newly
    ///   allocated node.
    fn split_leaf(&mut self, handle: &NodeHandle, idx: usize) -> (K, V, NodeHandle) {
        let mut node = self.get_node_mut(handle)
            .expect("node to split must exist");

        // we can only start splitting at leaf nodes
        debug_assert_eq!(node.edges(), 0);

        let mut right = Node::new();
        let k = node.keys[idx].take().expect("key must exist at split location");
        let v = node.vals[idx].take().expect("value must exist at split location");
        node.len -= 1;

        let from = idx + 1;
        for i in from..CAPACITY {
            let a = i - from;
            right.keys[a] = node.keys[i].take();
            right.vals[a] = node.vals[i].take();
            if let Some(_) = right.keys[a] {
                node.len -= 1;
                right.len += 1;
            }
        }

        let right_index = self.put(right);
        let right_handle = NodeHandle(right_index);
        (k, v, right_handle)
    }

    /// Splits the underlying node into three parts:
    ///
    /// - The node is truncated to only contain the edges and key/value pairs to the
    ///   right of this handle.
    /// - The key and value pointed to by this handle and extracted.
    /// - All the edges and key/value pairs to the right of this handle are put into
    ///   a newly allocated node.
    fn split_internal(&mut self, parent: u32, idx: usize) -> (K, V, NodeHandle) {
        let handle = NodeHandle(parent);
        let mut node = self.get_node_mut(&handle)
            .expect("node to split must exist");

        let count = node.len();
        let new_len = count - idx - 1;

        let mut right = Node::new();
        right.parent = Some(parent);
        right.parent_idx = Some(idx as u32);

        let k = node.keys[idx].take().expect("key must exist at split location");
        let v = node.vals[idx].take().expect("value must exist at split location");
        node.len -= 1;

        let from = idx + 1;
        for a in 0..new_len {
            let i = from + a;
            right.keys[a] = node.keys[i].take();
            right.vals[a] = node.vals[i].take();
            if let Some(_) = right.keys[a] {
                node.len -= 1;
                right.len += 1;
            }
        }
        for a in 0..new_len + 1 {
            let i = from + a;
            right.edges[a] = node.edges[i];
            node.edges[i] = None;
        }

        let right_index = self.put(right);
        let right_handle = NodeHandle(right_index);
        for i in (idx + 1)..(new_len + 1) {
            let handle = KVHandle::new(right_index, i as u32);
            self.correct_parent_link(handle);
        }

        (k, v, right_handle)
    }

    /// Adds a new internal node with a single edge, pointing to the previous root, and make that
    /// new node the root. This increases the height by 1 and is the opposite of `pop_level`.
    fn root_push_level(&mut self) -> NodeHandle {
        let handle = NodeHandle(self.header.root);
        let mut new_root = Node::<K, V>::new();
        new_root.edges[0] = Some(handle.0);
        let index = self.put(new_root);

        let mut old_root = self.get_node_mut(&self.header.root.into())
            .expect("root must exist when pushing level");
        old_root.parent = Some(index);
        old_root.parent_idx = None;

        self.header.root = index;
        NodeHandle(index)
    }

    pub fn insert_into_node(&mut self, handle: KVHandle, key: K, val: V) -> (InsertResult<K, V>, *mut V) {
        let node = self.get_node(&handle.into())
            .expect("node to insert into must exist");
        let len = node.len();

        if len < CAPACITY {
            let h = KVHandle::new(handle.node, search::search_linear(node, &key).0);
            let res = self.insert_fit(&h, key, val);
            self.header.len += 1;
            (InsertResult::Fit(handle), res)
        } else {
            let (k, v, right) = {
                //let (k, v, right) = self.split_leaf(&self.handle, B);
                let (k, v, right) = self.split_leaf(&handle.into(), B);
                (k, v, right)
            };

            let ptr = if handle.idx as usize <= B {
                // handle is left side
                self.insert_fit(&handle, key, val)
            } else {
                let h = KVHandle::new(right.0, handle.idx - (B as u32 + 1));
                self.insert_fit(&h, key, val)
            };

            (InsertResult::Split(handle, k, v, right), ptr)
        }
    }

    /// Insertinto parent with edge.node.
    fn insert_into_parent(&mut self, handle: KVHandle, key: K, val: V, edge: NodeHandle) -> InsertResult<K, V> {
        let node = self.get_node_mut(&handle.into())
            .expect("parent to insert into must exist");
        let len = node.len();

        if len < CAPACITY {
            let h = KVHandle::new(handle.node, search::search_linear(node, &key).0);
            self.insert_fit_edge(&h, key, val, edge);
            InsertResult::Fit(h)
        } else {
            let (k, v, right) = {
                let parent = handle.node;
                let (k, v, right) =
                    self.split_internal(parent, B);
                (k, v, right)
            };

            if handle.idx as usize <= B {
                // handle is left side
                self.insert_fit_edge(&handle, key, val, edge);
            } else {
                let h = KVHandle::new(right.0, handle.idx - (B as u32 + 1));
                self.insert_fit_edge(&h, key, val, edge);
            }

            InsertResult::Split(handle, k, v, right)
        }
    }

    /// Inserts a new key/value pair between the key/value pairs to the right and left of
    /// this edge. This method assumes that there is enough space in the node for the new
    /// pair to fit.
    ///
    /// The returned pointer points to the inserted value.
    fn insert_fit(&mut self, handle: &KVHandle, key: K, val: V) -> *mut V  {
        let mut node = self.get_node_mut(&handle.into())
            .expect("node to insert_fit into must exist");
        debug_assert!(node.len() < CAPACITY);

        let idx = handle.idx as usize;
        slice_insert(&mut node.keys, idx, Some(key));
        slice_insert(&mut node.vals, idx, Some(val));
        node.len += 1;

        node.vals[idx].as_mut().expect("value was just inserted at this position; qed")
    }

    /// Inserts a new key/value pair and an edge that will go to the right of that new pair
    /// between this edge and the key/value pair to the right of this edge. This method assumes
    /// that there is enough space in the node for the new pair to fit.
    fn insert_fit_edge(&mut self, handle: &KVHandle, key: K, val: V, edge: NodeHandle) {
        self.insert_fit(handle, key, val);

        let node = self.get_node_mut(&handle.into())
            .expect("node to insert (k, v, edge) into must exist");
        slice_insert(&mut node.edges, handle.idx() + 1, Some(edge.0));

        for idx in (handle.idx + 1)..(node.len() as u32 + 1) {
            let handle = KVHandle::new(handle.node, idx);
            self.correct_parent_link(handle);
        }
    }

    /// Fixes the parent pointer and index in the child node below this edge. This is useful
    /// when the ordering of edges has been changed, such as in the various `insert` methods.
    fn correct_parent_link(&mut self, handle: KVHandle) {
        let child = self.descend(&handle)
            .expect("child in which to correct parent link must exist");
        let mut node = self.get_node_mut(&child)
            .expect("child node must exist");
        node.parent = Some(handle.node);
        node.parent_idx = Some(handle.idx);
    }
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
    /// The latest vacant index.
    next_vacant: u32,
    /// The index of the root node.
    root: u32,
    /// The number of elements stored in the map.
    ///
    /// # Note
    ///
    /// We cannot simply use the underlying length of the vector
    /// since it would include vacant slots as well.
    len: u32,
    /// Number of nodes the BTree contains. This is not the number
    /// of elements!
    node_count: u32,
}

impl Flush for BTreeMapHeader {
    #[inline]
    fn flush(&mut self) {
        self.next_vacant.flush();
        self.root.flush();
        self.len.flush();
    }
}

/// A node in the BTree.
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct Node<K, V> {
    pub parent: Option<u32>,

    /// This node's index into the parent node's `edges` array.
    pub parent_idx: Option<u32>,

    /// The array storing the keys in the node.
    pub keys: [Option<K>; CAPACITY],

    /// The array storing the values in the node.
    pub vals: [Option<V>; CAPACITY],

    /// The pointers to the children of this node.
    pub edges: [Option<u32>; 2 * B],

    /// Elements in this node.
    pub len: u32,
}

impl<K, V> Node<K, V> {
    /// Creates a new `LeafNode`. Unsafe because all nodes should really be hidden behind
    /// `BoxedNode`, preventing accidental dropping of uninitialized keys and values.
    pub fn new() -> Self {
        Node {
            parent: None,
            parent_idx: None,
            keys: Default::default(),
            vals: Default::default(),
            edges: [None; 2 * B],
            len: 0,
        }
    }

    /// Returns the number of elements stored in the tree.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns the number of edges this node has.
    pub fn edges(&self) -> usize {
        self.edges
            .iter()
            .filter(|o| o.is_some())
            .count()
    }
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
        self.keys.flush();
        self.vals.flush();
        self.edges.flush();
    }
}

/// Points to a node in the tree.
#[derive(Debug)]
pub struct NodeHandle(pub u32);

impl From<&KVHandle> for NodeHandle {
    fn from(handle: &KVHandle) -> NodeHandle {
        NodeHandle(handle.node)
    }
}

impl From<KVHandle> for NodeHandle {
    fn from(handle: KVHandle) -> NodeHandle {
        NodeHandle(handle.node)
    }
}

impl From<u32> for NodeHandle {
    fn from(index: u32) -> NodeHandle {
        NodeHandle(index)
    }
}

/// Points to a key-value pair within a node in the tree.
#[derive(Clone, Copy, Debug)]
pub struct KVHandle {
    /// Index of the node in entries.
    pub node: u32,
    /// Index of the key-value pair within the node.
    pub idx: u32,
}

impl KVHandle {
    pub fn new(node: u32, idx: u32) -> Self {
        Self {
            node,
            idx,
        }
    }

    fn idx(&self) -> usize {
        self.idx as usize
    }
}

impl<K, V> Flush for BTreeMap<K, V>
where
    K: Encode + Flush,
    V: Encode + Flush,
{
    #[inline]
    fn flush(&mut self) {
        self.tree.flush();
    }
}

impl<K, V> Encode for BTreeMap<K, V> {
    fn encode_to<W: scale::Output>(&self, dest: &mut W) {
        self.tree.encode_to(dest);
    }
}

impl<K, V> Decode for BTreeMap<K, V> {
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        let tree = Tree::decode(input)?;
        Ok(Self { tree })
    }
}

impl<K, V> AllocateUsing for BTreeMap<K, V> {
    #[inline]
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
        where
            A: Allocate,
    {
        Self {
            tree: Tree::allocate_using(alloc),
        }
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
        self.tree.initialize();
    }
}

impl<K: Ord + core::fmt::Debug, V: core::fmt::Debug> BTreeMap<K, V> {
    /// Returns the number of elements stored in the map.
    pub fn len(&self) -> u32 {
        self.tree.len()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V> BTreeMap<K, V>
where
    K: Eq + Ord + scale::Codec  + core::fmt::Debug,
    V: scale::Codec  + core::fmt::Debug,
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
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    pub fn get(&self, key: &K) -> Option<&V> {
        match search::search_tree(&self.tree, &key) {
            SearchResult::Found(handle) => {
                let idx = handle.idx();
                let node = self.tree.get_node(&handle.into())
                    .expect("found node always exists; qed");
                let v = node.vals[idx].as_ref()
                    .expect("value must exist in found node; qed");
                Some(v)
            },
            SearchResult::GoDown(_) => None,
        }
    }

    /// Inserts a key-value pair into the map.
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
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map[&37], "c");
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.entry(key) {
            Entry::Occupied(mut entry) =>
                entry.insert(value),
            Entry::Vacant(entry) => {
                entry.insert(value);
                None
            }
        }
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    ///
    /// // count the number of occurrences of letters in the vec
    /// for x in vec!["a","b","a","c","a","b"] {
    ///     *count.entry(x).or_insert(0) += 1;
    /// }
    ///
    /// assert_eq!(count["a"], 3);
    /// ```
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        match search::search_tree(&self.tree, &key) {
            SearchResult::Found(handle) => {
                Entry::Occupied(OccupiedEntry {
                    tree: &mut self.tree,
                    //header: &mut self.header,
                    handle,
                })
            }
            SearchResult::GoDown(handle) => {
                Entry::Vacant(VacantEntry {
                    key: Some(key),
                    tree: &mut self.tree,
                    //header: &mut self.header,
                    handle,
                })
            }
        }
    }
}

impl<'a, K: Ord + Decode + Encode  + core::fmt::Debug, V: Decode + Encode  + core::fmt::Debug> Entry<'a, K, V> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
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
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
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
                LayoutField::of("entries", &self.tree),
            ],
        )
        .into()
    }
}

pub(crate) enum InsertResult<K, V> {
    Fit(KVHandle),
    Split(KVHandle, K, V, NodeHandle)
}

/// An entry within a BTreeMap collection.
///
/// This represents either an occupied entry with its associated value
/// or a vacant entry pointing to the next vacant entry.
#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
enum InternalEntry<K, V> {
    /// A vacant entry pointing to the next vacant index.
    Vacant(u32),
    /// An occupied entry containing the value.
    Occupied(Node<K, V>),
}

/// An entry of a storage map.
///
/// This can either store the entries key and value
/// or represent an entry that was removed after it
/// has been occupied with key and value.
#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub enum Entry<'a, K, V> {
    /// A vacant entry pointing to the next vacant index.
    Vacant(VacantEntry<'a, K, V>),
    /// An occupied entry containing the value.
    Occupied(OccupiedEntry<'a, K, V>),
}

/// An occupied entry of a storage map.
#[derive(Debug)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct OccupiedEntry<'a, K, V> {
    tree: &'a mut Tree<K, V>,
    handle: KVHandle,
}

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: Encode + Decode  + core::fmt::Debug + Ord,
    V: Encode + Decode  + core::fmt::Debug,
{
    /// Gets a reference to the key that would be used when inserting a value
    /// through the VacantEntry.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    pub fn key(&self) -> &K {
        self.key.as_ref().expect("entry does always have a key; qed")
    }

    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns a mutable reference to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    ///
    /// // count the number of occurrences of letters in the vec
    /// for x in vec!["a","b","a","c","a","b"] {
    ///     *count.entry(x).or_insert(0) += 1;
    /// }
    ///
    /// assert_eq!(count["a"], 3);
    /// ```
    pub fn insert(mut self, val: V) -> &'a mut V {
        let key = self.key.take()
            .expect("key is only taken when inserting, so must be there; qed");
        self.tree.insert(self.handle, key, val)
    }
}

impl<'a, K, V> OccupiedEntry<'a, K, V>
where
    K: Encode + Decode + Ord + core::fmt::Debug,
    V: Encode + Decode + core::fmt::Debug,
{
    /// Gets a reference to the key in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    /// map.entry("poneyland").or_insert(12);
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    pub fn key(&self) -> &K {
        self.kv().0
    }

    /// Gets a reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use std::collections::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     assert_eq!(o.get(), &12);
    /// }
    /// ```
    pub fn get(&self) -> &V {
        self.kv().1
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use std::collections::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
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
        self.kv_mut().1
    }

    /// Converts the entry into a mutable reference to its value.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    /// [`get_mut`]: #method.get_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use std::collections::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     *o.into_mut() += 10;
    /// }
    /// assert_eq!(map["poneyland"], 22);
    /// ```
    pub fn into_mut(self) -> &'a mut V {
        self.into_kv_mut().1
    }

    fn insert(&mut self, value: V) -> Option<V> {
        let node = self
            .tree
            .get_node_mut(&self.handle.into())
            .expect(
                "[ink_core::BTreeMap::take] Error: \
                         we already asserted that the entry at `n` exists",
            );
        node.vals[self.handle.idx()].replace(value)
    }

    fn kv(&self) -> (&K, &V) {
        let node = self.tree.get_node(&self.handle.into())
            .expect("node on OccupiedEntry must exist");
        let idx = self.handle.idx();
        let k = node.keys[idx].as_ref().expect("key must exist");
        let v = node.vals[idx].as_ref().expect("value must exist");
        (k, v)
    }

    fn kv_mut(&mut self) -> (&mut K, &mut V) {
        let idx = self.handle.idx();
        let node = self.tree.get_node_mut(&self.handle.into())
            .expect("node on OccupiedEntry must exist");
        let k = node.keys[idx].as_mut().expect("key must exist");
        let v = node.vals[idx].as_mut().expect("value must exist");
        (k, v)
    }

    fn into_kv_mut(self) -> (&'a mut K, &'a mut V) {
        let idx = self.handle.idx();
        let node = self.tree.get_node_mut(&self.handle.into())
            .expect("node to get kv pairs from must exist");
        let k = node.keys[idx].as_mut().expect("key must exist");
        let v = node.vals[idx].as_mut().expect("value must exist");
        (k, v)
    }
}

/// A vacant entry of a storage map.
#[derive(Debug)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct VacantEntry<'a, K, V> {
    // we use `Option<K`> to prevent running into partial move errors when putting.
    key: Option<K>,
    tree: &'a mut Tree<K, V>,
    handle: KVHandle,
}

impl<K, V> Flush for InternalEntry<K, V>
where
    K: Encode + Flush,
    V: Encode + Flush,
{
    #[inline]
    fn flush(&mut self) {
        match self {
            InternalEntry::Vacant(_) => (),
            InternalEntry::Occupied(occupied) => occupied.flush(),
        }
    }
}

fn slice_insert<T>(slice: &mut [T], idx: usize, val: T) {
    unsafe {
        ptr::copy(
            slice.as_ptr().add(idx),
            slice.as_mut_ptr().add(idx + 1),
            slice.len() - idx - 1
        );
        ptr::write(slice.get_unchecked_mut(idx), val);
    }
}

