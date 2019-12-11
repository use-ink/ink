// Copyright 2018-2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.node (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.node
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
use core::{cmp::Ord, ptr};
use super::search::{self, SearchResult};
use std::borrow::Borrow;

/// Each node in the tree has 2 * B children.
pub(in super) const B: usize = 6;

/// Number of elements which can be stored in one node of the tree.
pub const CAPACITY: usize = 2 * B - 1;

/// Mapping stored in the contract storage.
///
/// ToDo add more verbose comments
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct BTreeMap<K, V> {
    /// Stores densely packed general BTreeMap information.
    header: storage::Value<BTreeMapHeader>,
    /// The entries of the map.
    entries: SyncChunk<InternalEntry<K, V>>
}

impl<K, V> Flush for BTreeMap<K, V>
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

impl<K, V> Encode for BTreeMap<K, V> {
    fn encode_to<W: scale::Output>(&self, dest: &mut W) {
        self.header.encode_to(dest);
        self.entries.encode_to(dest);
    }
}

impl<K, V> Decode for BTreeMap<K, V> {
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        let header = storage::Value::decode(input)?;
        let entries = SyncChunk::decode(input)?;
        Ok(Self { header, entries })
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
            entries: SyncChunk::allocate_using(alloc),
        }
    }
}

impl<K, V> BTreeMap<K, V>
{
    pub fn root(&self) -> Option<u32> {
        self.header.root
    }
}

impl<K, V> BTreeMap<K, V>
where
    K: Encode + Decode + Ord,
    V: Encode + Decode,
{
    pub fn kv_mut(&mut self, handle: &KVHandle) -> (&mut K, &mut V) {
        let idx = handle.idx();
        let node = self.get_node_mut(&handle.into())
            .expect("node on OccupiedEntry must exist");
        let k = node.keys[idx].as_mut().expect("key must exist");
        let v = node.vals[idx].as_mut().expect("value must exist");
        (k, v)
    }

    /// Returns the `HandleType` of `handle`. Either `Leaf` or `Internal`.
    pub(in super) fn get_handle_type(&self, handle: &NodeHandle) -> HandleType {
        let node = self.get_node(handle).expect("node must exist");
        let children = node.edges();
        if children == 0 {
            HandleType::Leaf
        } else {
            HandleType::Internal
        }
    }

    pub fn first_edge(&self, handle: &NodeHandle) -> KVHandle {
        KVHandle::new(handle.node, 0)
    }

    pub fn last_edge(&self, handle: &NodeHandle) -> KVHandle {
        let node = self.get_node(handle).expect("node must exist");
        let len = node.len();
        KVHandle::new(handle.node, len as u32)
    }

    fn first_leaf_edge(&self, mut handle: NodeHandle) -> KVHandle {
        loop {
            match self.get_handle_type(&handle) {
                HandleType::Leaf => return self.first_edge(&handle),
                HandleType::Internal => {
                    let first_edge = self.first_edge(&handle);
                    handle = self.descend(&first_edge).expect("must exist");
                }
            }
        }
    }

    /// Left edge from this node.
    fn left_edge(&self, handle: &KVHandle) -> KVHandle {
        KVHandle::new(handle.node, handle.idx)
    }

    /// Right edge from this node.
    fn right_edge(&self, handle: &KVHandle) -> KVHandle {
        KVHandle::new(handle.node, handle.idx + 1)
    }

    pub fn left_kv(&self, handle: KVHandle) -> Result<KVHandle, KVHandle> {
        if handle.idx > 0 {
            Ok(KVHandle::new(handle.node, handle.idx - 1))
        } else {
            Err(handle)
        }
    }

    pub fn right_kv(&self, handle: KVHandle) -> Result<KVHandle, KVHandle> {
        let node = self.get_node(&handle.into())
            .expect("node to descend from must exist; qed");
        if handle.idx < node.len() as u32 {
            Ok(KVHandle::new(handle.node, handle.idx))
        } else {
            Err(handle)
        }
    }

    pub fn remove_kv(&mut self, handle: KVHandle) -> (K, V) {
        self.header.len -= 1;

        let (small_leaf, old_key, old_val, mut new_len) = match self.get_handle_type(&handle.into()) {
            HandleType::Leaf => self.remove_handle(&handle),
            HandleType::Internal => {
                let to_remove = {
                    let right = self.right_edge(&handle);
                    let child = self.descend(&right)
                        .expect("every internal node has children; qed");
                    let first_leaf = self.first_leaf_edge(child);
                    let to_remove = self.right_kv(first_leaf)
                        .ok().expect("right_kv must exist");
                    to_remove
                };

                let (hole, key, val, nl) = self.remove_handle(&to_remove);
                let node = self.get_node_mut(&handle.into())
                    .expect("node must exist");
                let idx = handle.idx as usize;
                let old_key = node.keys[idx].replace(key).expect("handle must be valid");
                let old_val = node.vals[idx].replace(val).expect("handle must be valid");
                (hole, old_key, old_val, nl)
            }
        };

        // Handle underflow
        let mut what = small_leaf.into();
        while new_len < CAPACITY / 2 {
            match self.handle_underfull_node(&what) {
                UnderflowResult::AtRoot => break,
                UnderflowResult::EmptyParent(_) => unreachable!(),
                UnderflowResult::Merged(parent) => {
                    let parentn = self.get_node(&parent)
                        .expect("parent node must exist");
                    // ToDo is this correct? why not check if parent == None?
                    if parentn.len() == 0 {
                        // We must be at the root
                        self.root_pop_level();
                        break;
                    } else {
                        what = parent;
                        new_len = parentn.len();
                    }
                }
                UnderflowResult::Stole(_) => break,
            }
        }

        if new_len == 0 {
            // TODO maybe check if there are edges? and if so pop root?
            self.remove_node(what);
            self.header.root = None;
            self.header.next_vacant = None;
        }

        (old_key, old_val)
    }

    fn handle_underfull_node(&mut self, node: &NodeHandle) -> UnderflowResult {
        let parent = if let Some(parent) = self.ascend(node){
            parent
        } else {
            return UnderflowResult::AtRoot;
        };

        let (is_left, handle) = match self.left_kv(parent) {
            Ok(left) => (true, left),
            Err(parent) => {
                match self.right_kv(parent) {
                    Ok(right) => (false, right),
                    Err(parent) => {
                        return UnderflowResult::EmptyParent(parent.into());
                    }
                }
            }
        };

        if self.can_merge(&handle) {
            UnderflowResult::Merged(self.merge(&handle).into())
        } else {
            if is_left {
                self.steal_left(&handle);
            } else {
                self.steal_right(&handle);
            }
            UnderflowResult::Stole(handle.into())
        }
    }

    /// Returns `true` if it is valid to call `.merge()`, i.e., whether there is enough room in
    /// a node to hold the combination of the nodes to the left and right of this handle along
    /// with the key/value pair at this handle.
    pub fn can_merge(&self, handle: &KVHandle) -> bool {
        let len_left = self.left_child_node(handle).expect("left child must exist").len();
        let len_right = self.right_child_node(handle).expect("right child must exist").len();

        len_left + len_right + 1 <= CAPACITY
    }

    /// Combines the node immediately to the left of this handle, the key/value pair pointed
    /// to by this handle, and the node immediately to the right of this handle into one new
    /// child of the underlying node, returning an edge referencing that new child.
    ///
    /// Assumes that this edge `.can_merge()`.
    pub fn merge(&mut self, handle: &KVHandle) -> KVHandle {
        let right_child = self.right_child(&handle).expect("right child must exist");
        let right_node = self.get_node(&right_child).expect("right child must exist");
        let right_edges = right_node.edges.as_ptr();
        let right_keys = right_node.keys.as_ptr();
        let right_vals = right_node.vals.as_ptr();
        let right_len = right_node.len();

        let node = self.get_node_mut(&handle.into())
            .expect("node must exist");
        unsafe {
            slice_remove(&mut node.edges, handle.idx as usize + 1);
        }
        let removed_key = unsafe {
            slice_remove(&mut node.keys, handle.idx as usize)
        };
        let removed_val = unsafe {
            slice_remove(&mut node.vals, handle.idx as usize)
        };
        let node_len = node.len();

        let left_child = self.left_child(handle).expect("left child must exist");
        let left_node = self.get_node_mut(&left_child).expect("left child must exist");
        let left_len = left_node.len();
        debug_assert!(left_len + right_len + 1 <= CAPACITY);

        unsafe {
            ptr::write(left_node.keys.get_unchecked_mut(left_len), removed_key);
            ptr::copy_nonoverlapping(
                right_keys,
                left_node.keys.as_mut_ptr().add(left_len + 1),
                right_len
            );
            ptr::write(left_node.vals.get_unchecked_mut(left_len), removed_val);
            ptr::copy_nonoverlapping(
                right_vals,
                left_node.vals.as_mut_ptr().add(left_len + 1),
                right_len
            );
        }

        for i in handle.idx + 1..node_len as u32 {
            let h = KVHandle::new(handle.node, i);
            self.correct_parent_link(h);
        }

        let node = self.get_node_mut(&handle.into()).expect("merge 308");
        node.len -= 1;

        let left_node = self.get_node_mut(&left_child).expect("left child must exist");
        left_node.len += right_len as u32 + 1;

        // if the direct children have more children we need to take care of those
        if self.has_children(&left_child) || self.has_children(&right_child) {
            let left_node = self.get_node_mut(&left_child).expect("313");
            unsafe {
                ptr::copy_nonoverlapping(
                    right_edges,
                    left_node
                        .edges
                        .as_mut_ptr()
                        .add(left_len + 1),
                    right_len + 1
                );
            }

            for i in left_len+1..left_len+right_len+2 {
                let h = KVHandle::new(
                    left_child.node,
                    i as u32
                );
                self.correct_parent_link(h);
            }
        }

        self.remove_node(right_child);
        KVHandle::new(handle.node, handle.idx)
    }

    /// Removes a key/value pair from the end of this node. If this is an internal node,
    /// also removes the edge that was to the right of that pair.
    pub fn pop(&mut self, handle: NodeHandle) -> (K, V, Option<NodeHandle>) {
        let typ = {
            self.get_handle_type(&handle)
        };
        let (key, val, idx) = {
            let node = self.get_node_mut(&handle).expect("node must exist");
            // Necessary for correctness, but this is an internal module
            debug_assert!(node.len() > 0);
            let idx = node.len() - 1;
            let key = node.keys[idx].take().expect("key must exist");
            let val = node.vals[idx].take().expect("val must exist");
            node.len -= 1;
            (key, val, idx)
        };
        let edge = match typ {
            HandleType::Leaf => None,
            HandleType::Internal => {
                let edge = {
                    let node = self.get_node_mut(&handle).expect("node must exist");
                    node.edges[idx + 1].take().expect("edge must exist")
                };
                self.set_parent(&NodeHandle::new(edge), None, None);
                Some(NodeHandle::new(edge))
            }
        };

        (key, val, edge)
    }

    /// This removes a key/value pair from the left child and replaces it with the key/value pair
    /// pointed to by this handle while pushing the old key/value pair of this handle into the right
    /// child.
    pub fn steal_left(&mut self, handle: &KVHandle) {
        let left_child = self.left_child(handle).expect("left child must exist");
        let (k, v, edge) = self.pop(left_child);

        let node = self.get_node_mut(&handle.into()).expect("417");
        let k = node.keys[handle.idx as usize].replace(k).expect("477");
        let v = node.vals[handle.idx as usize].replace(v).expect("478");

        let right = self.right_edge(handle);
        let child = self.descend(&right).expect("457");
        match self.get_handle_type(&child) {
            HandleType::Leaf => self.push_front_leaf(&child, k, v),
            HandleType::Internal => self.push_front_internal(&child, k, v, edge.unwrap())
        }
    }

    /// Removes a key/value pair from the beginning of this node. If this is an internal node,
    /// also removes the edge that was to the left of that pair.
    pub fn pop_front(&mut self, handle: &NodeHandle) -> (K, V, Option<NodeHandle>) {
        let typ = self.get_handle_type(handle);
        let node = self.get_node_mut(handle).expect("417");

        // Necessary for correctness, but this is an internal module
        debug_assert!(node.len() > 0);
        let old_len = node.len();

        let key = unsafe { slice_remove(&mut node.keys, 0).expect("key must exist") };
        let val = unsafe { slice_remove(&mut node.vals, 0).expect("val must exist") };

        let edge = match typ {
            HandleType::Leaf => None,
            HandleType::Internal => {
                let edge = unsafe {
                    slice_remove(&mut node.edges, 0).expect("edge must exist")
                };

                let new_root = NodeHandle::new(edge);
                self.set_parent(&new_root, None, None);

                for i in 0..old_len {
                    let h = KVHandle::new(handle.node, i as u32);
                    self.correct_parent_link(h);
                }

                Some(new_root)
            }
        };

        let node = self.get_node_mut(handle).expect("node must exist");
        node.len -= 1;

        (key, val, edge)
    }

    /// Adds a key/value pair to the beginning of the node.
    pub fn push_front_leaf(&mut self, handle: &NodeHandle, key: K, val: V) {
        let node = self.get_node_mut(handle).expect("417");
        // Necessary for correctness, but this is an internal module
        debug_assert!(node.len() < CAPACITY);

        unsafe {
            slice_insert(&mut node.keys, 0, Some(key));
            slice_insert(&mut node.vals, 0, Some(val));
        }
        node.len += 1;
    }

    /// Adds a key/value pair and an edge to go to the left of that pair to
    /// the beginning of the node.
    pub fn push_front_internal(&mut self, handle: &NodeHandle, key: K, val: V, edge: NodeHandle) {
        let node = self.get_node_mut(handle).expect("417");
        // Necessary for correctness, but this is an internal module
        //debug_assert!(self.header.height == self.height - 1);
        debug_assert!(node.len() < CAPACITY);

        unsafe {
            slice_insert(&mut node.keys, 0, Some(key));
            slice_insert(&mut node.vals, 0, Some(val));
            slice_insert(&mut node.edges, 0, Some(edge.node));
        }

        node.len += 1;

        self.correct_all_childrens_parent_links(handle);
    }

    /// This removes a key/value pair from the right child and replaces it with the key/value pair
    /// pointed to by this handle while pushing the old key/value pair of this handle into the left
    /// child.
    pub fn steal_right(&mut self, handle: &KVHandle) {
        let right = self.right_edge(handle);
        let child = self.descend(&right).expect("515");
        let (k, v, edge) = self.pop_front(&child);

        let (k, v) = {
            let node = self.get_node_mut(&handle.into()).expect("417");
            let k = node.keys[handle.idx as usize].replace(k).expect("477");
            let v = node.vals[handle.idx as usize].replace(v).expect("478");
            (k, v)
        };

        let left_child = self.left_child(handle).expect("left child must exist");
        match self.get_handle_type(&left_child) {
            HandleType::Leaf => self.push_leaf(&left_child, k, v),
            HandleType::Internal => {
                self.push_internal(left_child, k, v, edge.unwrap())
            }
        }
    }

    fn remove_node(&mut self, handle: NodeHandle) {
        let n = handle.node;
        let _ = match self.entries.get(n) {
            None | Some(InternalEntry::Vacant(_)) => None,
            Some(InternalEntry::Occupied(_)) => {
                match self
                    .entries
                    .put(n, InternalEntry::Vacant(self.header.next_vacant))
                    .expect(
                        "[ink_core::BTreeMap::take] Error: \
                         we already asserted that the entry at `n` exists",
                    ) {
                    InternalEntry::Occupied(val) => {
                        // When removing a node set `next_vacant` to this node index
                        self.header.next_vacant = Some(n);
                        self.header.node_count -= 1;
                        Some(val)
                    }
                    InternalEntry::Vacant(_) => {
                        unreachable!(
                            "[ink_core::BTreeMap::take] Error: \
                             we already asserted that the entry is occupied"
                        )
                    }
                }
            }
        };
    }

    /// Removes the key/value pair pointed to by this handle, returning the edge between the
    /// now adjacent key/value pairs to the left and right of this handle.
    fn remove_handle(&mut self, handle: &KVHandle) -> (KVHandle, K, V, usize) {
        let node = self.get_node_mut(&handle.into())
            .expect("node must exist");
        let k = unsafe {
            slice_remove(&mut node.keys, handle.idx as usize)
                .expect("key must exist")
        };
        let v = unsafe {
            slice_remove(&mut node.vals, handle.idx as usize)
                .expect("val must exist")
        };

        node.len -= 1;
        let nl = node.len as usize;
        (self.left_edge(handle), k, v, nl)
    }

    /// ToDo
    pub fn insert_kv<'a>(&'a mut self, handle: KVHandle, key: K, val: V) -> &'a mut V {
        if self.len() == 0 && self.root().is_none() {
            let mut node = Node::<K, V>::new();
            node.keys[0] = Some(key);
            node.vals[0] = Some(val);
            node.len = 1;

            let index = self.put(node);
            self.header.len += 1;
            self.header.root = Some(index);

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
                self.ascend(&NodeHandle::new(left.node))
            }
        };

        loop {
            match cur_parent {
                Some(handle) => {
                    let parent = handle;
                    match self.insert_into_parent(parent, ins_k, ins_v, ins_edge) {
                        InsertResult::Fit(_) => {
                            self.header.len += 1;
                            return unsafe { &mut *out_ptr };
                        },
                        InsertResult::Split(left, k, v, right) => {
                            ins_k = k;
                            ins_v = v;
                            ins_edge = right;
                            cur_parent = self.ascend(&NodeHandle::new(left.node));
                        }
                    }
                }
                None => {
                    let new_root = self.root_push_level();
                    self.header.len += 1;
                    self.push_internal(new_root, ins_k, ins_v, ins_edge);
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
        node.edges[handle.idx()].map(|e| NodeHandle::new(e))
    }

    /// Finds the node pointed to by this edge.
    ///
    /// `edge.descend().ascend().unwrap()` and `node.ascend().unwrap().descend()` should
    /// both, upon success, do nothing.
    pub fn ascend(&self, handle: &NodeHandle) -> Option<KVHandle> {
        let node = self.get_node(handle)
            .expect("node to ascend from must exist");

        node.parent.map(|parent| {
            let idx = node.parent_idx
                .expect("if parent exists, parent_idx always exist as well; qed");
            KVHandle::new(parent, idx)
        })
    }

    /// Fetches a reference to the node behind the supplied handle.
    pub(in super) fn get_node(&self, handle: &NodeHandle) -> Option<&Node<K, V>> {
        let entry = self.entries.get(handle.node)?;
        match entry {
            InternalEntry::Occupied(occupied) => Some(occupied),
            InternalEntry::Vacant(_) => None,
        }
    }

    pub(in super) fn into_kv(&self, handle: &KVHandle) -> (&K, &V) {
        let node = self.get_node(&handle.into())
            .expect("node on OccupiedEntry must exist");
        let k = node.keys[handle.idx()].as_ref().expect("key must exist");
        let v = node.vals[handle.idx()].as_ref().expect("value must exist");
        (k, v)
    }

    /// Fetches a mutable reference to the node behind the supplied handle.
    fn get_node_mut(&mut self, handle: &NodeHandle) -> Option<&mut Node<K, V>> {
        let entry = self.entries.get_mut(handle.node)?;
        match entry {
            InternalEntry::Occupied(occupied) => Some(occupied),
            InternalEntry::Vacant(_) => None,
        }
    }

    /// Put the element into the tree at the next vacant position.
    ///
    /// Returns the tree index that the element was put into.
    fn put(&mut self, node: Node<K, V>) -> u32 {
        let node_index = match self.header.next_vacant {
            None => {
                // then there is no vacant entry which we can reuse
                self.entries.set(self.header.node_count, InternalEntry::Occupied(node));
                self.header.node_count
            },
            Some(current_vacant) => {
                // then there is a vacant entry which we can reuse
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
                // when putting node set next_vacant to the next_vacant which was found here
                self.header.next_vacant = next_vacant;
                current_vacant
            }
        };
        self.header.node_count += 1;
        node_index
    }

    /// Adds a key/value pair and an edge to go to the right of that pair to
    /// the end of the node.
    fn push_internal(&mut self, dst: NodeHandle, key: K, val: V, edge: NodeHandle) {
        let node = self.get_node_mut(&dst)
            .expect("destination node to push to must exist");
        node.keys[node.len()] = Some(key);
        node.vals[node.len()] = Some(val);
        node.edges[node.len() + 1] = Some(edge.node);

        let handle = KVHandle::new(dst.node, node.len() as u32 + 1);
        node.len += 1;
        self.correct_parent_link(handle);
    }

    /// Adds a key/value pair the end of the node.
    pub fn push_leaf(&mut self, handle: &NodeHandle, key: K, val: V) {
        let mut node = self.get_node_mut(handle)
            .expect("destination node to push to must exist");

        debug_assert!(node.len() < CAPACITY);

        let idx = node.len();
        node.keys[idx] = Some(key);
        node.vals[idx] = Some(val);
        node.len += 1;
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
        let right_handle = NodeHandle::new(right_index);
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
        let handle = NodeHandle::new(parent);
        let node = self.get_node_mut(&handle)
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
            right.edges[a] = node.edges[i].take();
        }

        let right_index = self.put(right);
        let right_handle = NodeHandle::new(right_index);
        for i in 0..(new_len + 1) {
            let handle = KVHandle::new(right_index, i as u32);
            self.correct_parent_link(handle);
        }

        (k, v, right_handle)
    }

    /// Adds a new internal node with a single edge, pointing to the previous root, and make that
    /// new node the root. This increases the height by 1 and is the opposite of `pop_level`.
    fn root_push_level(&mut self) -> NodeHandle {
        let handle = NodeHandle::new(self.header.root.expect("855"));

        let mut new_root = Node::<K, V>::new();
        new_root.edges[0] = Some(handle.node);
        let index = self.put(new_root);

        let mut old_root = self.get_node_mut(&self.header.root.expect("861").into())
            .expect("root must exist when pushing level");
        old_root.parent = Some(index);
        old_root.parent_idx = Some(0);

        self.header.root = Some(index);
        NodeHandle::new(index)
    }

    /// Removes the root node, using its first child as the new root. This cannot be called when
    /// the tree consists only of a leaf node. As it is intended only to be called when the root
    /// has only one edge, no cleanup is done on any of the other children are elements of the root.
    /// This decreases the height by 1 and is the opposite of `push_level`.
    pub fn root_pop_level(&mut self) {
        //debug_assert!(node.edges() == 1);

        let handle = NodeHandle::new(self.header.root.expect("879"));
        let edge = self.first_edge(&handle);

        let child = self.descend(&edge).expect("680");
        self.set_parent(&child, None, None);

        self.header.root = Some(child.node);

        self.remove_node(handle);
    }

    fn set_parent(&mut self, handle: &NodeHandle, node_id: Option<u32>, idx: Option<u32>) {
        let node = self.get_node_mut(handle).expect("node must exist");
        node.parent = node_id;
        node.parent_idx = idx;
    }

    fn insert_into_node(&mut self, handle: KVHandle, key: K, val: V) -> (InsertResult<K, V>, *mut V) {
        let node = self.get_node(&handle.into())
            .expect("node to insert into must exist");
        let len = node.len();

        if len < CAPACITY {
            let h = KVHandle::new(handle.node, search::search_linear(node, &key).0);
            let res = self.insert_fit(&h, key, val);
            self.header.len += 1;
            (InsertResult::Fit(handle), res)
        } else {
            let (k, v, right) = self.split_leaf(&handle.into(), B);

            let ptr = if handle.idx as usize <= B {
                // handle is left side
                self.insert_fit(&handle, key, val)
            } else {
                let h = KVHandle::new(right.node, handle.idx - (B as u32 + 1));
                self.insert_fit(&h, key, val)
            };

            (InsertResult::Split(handle, k, v, right), ptr)
        }
    }

    /// Insert into parent with edge.node.
    fn insert_into_parent(&mut self, handle: KVHandle, key: K, val: V, edge: NodeHandle) -> InsertResult<K, V> {
        let node = self.get_node_mut(&handle.into())
            .expect("parent to insert into must exist");
        let len = node.len();

        if len < CAPACITY {
            let h = KVHandle::new(handle.node, search::search_linear(node, &key).0);
            self.insert_fit_edge(&h, key, val, edge);
            InsertResult::Fit(h)
        } else {
            let (k, v, right) = self.split_internal(handle.node, B);
            if handle.idx as usize <= B {
                // handle is left side
                self.insert_fit_edge(&handle, key, val, edge);
            } else {
                let h = KVHandle::new(right.node, handle.idx - (B as u32 + 1));
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
        unsafe {
            slice_insert(&mut node.keys, idx, Some(key));
            slice_insert(&mut node.vals, idx, Some(val));
        }
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

       unsafe {
           slice_insert(&mut node.edges, handle.idx() + 1, Some(edge.node));
       }

        for idx in (handle.idx + 1)..(node.len() as u32 + 1) {
            let handle = KVHandle::new(handle.node, idx);
            self.correct_parent_link(handle);
        }
    }

    fn left_child_node(&self, handle: &KVHandle) -> Option<&Node<K, V>> {
        let child = self.left_child(handle)?;
        self.get_node(&child)
    }

    fn right_child_node(&self, handle: &KVHandle) -> Option<&Node<K, V>> {
        let child = self.right_child(handle)?;
        self.get_node(&child)
    }

    fn left_child(&self, handle: &KVHandle) -> Option<NodeHandle> {
        let left = self.left_edge(handle);
        self.descend(&left)
    }

    fn right_child(&self, handle: &KVHandle) -> Option<NodeHandle> {
        let right = self.right_edge(handle);
        self.descend(&right)
    }

    /// Fixes the parent pointer and index in the child node below this edge. This is useful
    /// when the ordering of edges has been changed, such as in the various `insert` methods.
    fn correct_parent_link(&mut self, handle: KVHandle) {
        let child = self.descend(&handle)
            .expect("child in which to correct parent link must exist");
        self.set_parent(&child, Some(handle.node), Some(handle.idx));
    }

    fn correct_childrens_parent_links(&mut self, handle: &NodeHandle, first: usize, after_last: usize) {
        for i in first..after_last {
            let h = KVHandle::new(handle.node, i as u32);
            self.correct_parent_link(h);
        }
    }

    fn correct_all_childrens_parent_links(&mut self, handle: &NodeHandle) {
        let node = self.get_node(handle).expect("node must exist");
        let len = node.len();
        self.correct_childrens_parent_links(handle, 0, len + 1);
    }

    fn has_children(&mut self, handle: &NodeHandle) -> bool {
        let node = self.get_node(handle).expect("node must exist");
        node.edges() > 0
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
#[derive(Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub(in super) struct BTreeMapHeader {
    /// The latest vacant index.
    next_vacant: Option<u32>,
    /// The index of the root node.
    root: Option<u32>,
    /// The number of elements stored in the map.
    ///
    /// # Note
    ///
    /// We cannot simply use the underlying length of the vector
    /// since it would include vacant slots as well.
    len: u32,
    /// Number of nodes the BTree contains. This is not the number
    /// of elements!
    pub(in super) node_count: u32,
}

impl Flush for BTreeMapHeader {
    #[inline]
    fn flush(&mut self) {
        self.next_vacant.flush();
        self.root.flush();
        self.len.flush();
        self.node_count.flush();
    }
}

/// A node in the BTree.
#[derive(PartialEq, Eq, scale::Encode, scale::Decode)]
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
        self.len.flush();
    }
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

/// Points to a node in the tree.
pub struct NodeHandle {
    node: u32,
}

impl NodeHandle {
    pub fn new(node: u32) -> Self {
        Self {
            node,
        }
    }

    pub fn node(&self) -> u32 {
        self.node
    }
}

impl From<&KVHandle> for NodeHandle {
    fn from(handle: &KVHandle) -> NodeHandle {
        NodeHandle::new(handle.node)
    }
}

impl From<KVHandle> for NodeHandle {
    fn from(handle: KVHandle) -> NodeHandle {
        NodeHandle::new(handle.node)
    }
}

impl From<u32> for NodeHandle {
    fn from(index: u32) -> NodeHandle {
        NodeHandle::new(index)
    }
}

/// Points to a key-value pair within a node in the tree.
#[derive(Clone, Copy)]
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

    /*
    /// Todo
    pub(crate) fn entries(&self) -> SyncChunk<InternalEntry<K, V>> {
        *self.entries
    }
    */

    #[cfg(test)]
    pub(in super) fn header(&self) -> &BTreeMapHeader {
        &*self.header
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V> BTreeMap<K, V>
where
    K: Eq + Ord + scale::Codec,
    V: scale::Codec,
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
    /// use ink_core::storage::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
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
            SearchResult::Found(handle) => Some(self.into_kv(&handle).1),
            SearchResult::NotFound(_) => None,
        }
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// The supplied key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ink_core::collections::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
    /// assert_eq!(map.get_key_value(&2), None);
    /// ```
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where K: Borrow<Q>,
          Q: Ord
    {
        match search::search_tree(&self, key) {
            SearchResult::Found(handle) => Some(self.into_kv(&handle)),
            SearchResult::NotFound(_) => None,
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
    /// ```no_run
    /// use ink_core::collections::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where K: Borrow<Q>,
          Q: Ord
    {
        self.get(key).is_some()
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
    /// use ink_core::storage::BTreeMap;
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
            Entry::Occupied(mut entry) => {
                entry.insert(value)
            },
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
    /// ```
    /// use ink_core::storage::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
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
            SearchResult::Found(handle) => {
                let o = OccupiedEntry {
                    tree:  self,
                    handle,
                };
                Some(o.remove())
            }
            SearchResult::NotFound(_) => None,
        }
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use ink_core::storage::BTreeMap;
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
        match search::search_tree(&self, &key) {
            SearchResult::Found(handle) => {
                Entry::Occupied(OccupiedEntry {
                    tree: self,
                    handle,
                })
            }
            SearchResult::NotFound(handle) => {
                Entry::Vacant(VacantEntry {
                    key: Some(key),
                    tree: self,
                    handle,
                })
            }
        }
    }
}

impl<'a, K: Ord + Decode + Encode, V: Decode + Encode> Entry<'a, K, V> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use ink_core::storage::BTreeMap;
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
    /// use ink_core::storage::BTreeMap;
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

pub(in super) enum InsertResult<K, V> {
    Fit(KVHandle),
    Split(KVHandle, K, V, NodeHandle)
}

enum UnderflowResult {
    AtRoot,
    EmptyParent(NodeHandle),
    Merged(NodeHandle),
    Stole(NodeHandle),
}

/// The node type, either a `Leaf` (a node without children) or
/// `Internal` (a node with children).
pub(in super) enum HandleType {
    Leaf,
    Internal,
}

/// An entry within a BTreeMap collection.
///
/// This represents either an occupied entry with its associated value
/// or a vacant entry pointing to the next vacant entry.
#[derive(Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
enum InternalEntry<K, V> {
    /// A vacant entry pointing to the next vacant index.
    Vacant(Option<u32>),
    /// An occupied entry containing the value.
    Occupied(Node<K, V>),
}

/// An entry of a storage map.
///
/// This can either store the entries key and value
/// or represent an entry that was removed after it
/// has been occupied with key and value.
#[derive(Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub enum Entry<'a, K, V> {
    /// A vacant entry pointing to the next vacant index.
    Vacant(VacantEntry<'a, K, V>),
    /// An occupied entry containing the value.
    Occupied(OccupiedEntry<'a, K, V>),
}

/// An occupied entry of a storage map.
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
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
    /// ```
    /// use ink_core::storage::BTreeMap;
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
    /// ```no_run
    /// use ink_core::storage::BTreeMap;
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
        self.tree.insert_kv(self.handle, key, val)
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
    /// ```
    /// use ink_core::storage::BTreeMap;
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
    /// use ink_core::storage::BTreeMap;
    /// use ink_core::storage::btree_map::Entry;
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
    /// use ink_core::storage::BTreeMap;
    /// use ink_core::storage::btree_map::Entry;
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
    /// use ink_core::storage::BTreeMap;
    /// use ink_core::storage::btree_map::Entry;
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

    /// Takes the value of the entry out of the map, and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// use ink_core::storage::BTreeMap;
    /// use ink_core::storage::btree_map::Entry;
    ///
    /// let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     assert_eq!(o.remove(), 12);
    /// }
    /// // If we try to get "poneyland"'s value, it'll panic:
    /// // println!("{}", map["poneyland"]);
    /// ```
    pub fn remove(self) -> V {
        self.tree.remove_kv(self.handle).1
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
        self.tree.into_kv(&self.handle)
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
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct VacantEntry<'a, K, V> {
    // we use `Option<K`> to prevent running into partial move errors when putting.
    key: Option<K>,
    tree: &'a mut BTreeMap<K, V>,
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
            InternalEntry::Vacant(_) => (), // ToDo why not flushing?
            InternalEntry::Occupied(occupied) => occupied.flush(),
        }
    }
}

unsafe fn slice_insert<T>(slice: &mut [T], idx: usize, val: T) {
    ptr::copy(
        slice.as_ptr().add(idx),
        slice.as_mut_ptr().add(idx + 1),
        slice.len() - idx - 1
    );
    ptr::write(slice.get_unchecked_mut(idx), val);
}

unsafe fn slice_remove<T>(slice: &mut [Option<T>], idx: usize) -> Option<T> {
    let ret = ptr::read(slice.get_unchecked(idx));
    let cnt = slice.len() - idx - 1;
    ptr::copy(
        slice.as_ptr().add(idx + 1),
        slice.as_mut_ptr().add(idx),
        cnt
    );

    // clear
    ptr::write(
        slice.as_mut_ptr().add(idx + cnt),
        None
    );

    ret
}
