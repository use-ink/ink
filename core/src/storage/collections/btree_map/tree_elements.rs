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

/// A node in the tree.
///
/// Each node contains `CAPACITY` keys and values and an edges array over
/// which children nodes can be linked. Each child has a link to its parent.
///
/// Each node is stored as one storage entity. This reduces storage access,
/// since with each fetch the entire content of a node (all its elements, etc.)
/// are fetched.
#[derive(PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub(super) struct Node<K, V> {
    /// A reference to this node's parent node.
    parent: Option<NodeHandle>,

    /// This node's index into the parent node's `edges` array.
    /// If, for example, `parent_idx = Some(2)` this refers to the
    /// second position in the `edges` array of its parent node.
    parent_idx: Option<u32>,

    /// The array storing a pointer to the key/value pairs in a node.
    /// For performance reasons each node contains only a pointer to the index where
    /// the key/value pair is stored in `kv_pairs`. Otherwise re-balancing the tree
    /// or moving entries would be very expensive. With this indirection we only
    /// have to move the `u32` index.
    pairs: [Option<KVStoragePointer>; CAPACITY],

    /// The pointers to the children of this node.
    edges: [Option<u32>; EDGES],

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
    /// Returns the number of elements (i.e. key/value pairs) stored in this node.
    pub(super) fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns the edges stored in this node.
    #[cfg(test)]
    pub(super) fn edges(&self) -> &[Option<u32>; EDGES] {
        &self.edges
    }

    /// Create a new `Node`. The node is empty and all fields are instantiated with `None`.
    fn new() -> Self {
        Node {
            parent: None,
            parent_idx: None,
            pairs: Default::default(),
            edges: [None; 2 * B],
            len: 0,
            marker: Default::default(),
        }
    }

    /// Returns the number of edges (i.e. children) this node has.
    fn set_len(&mut self, new_len: usize) {
        self.len = new_len as u32;
    }

    /// Returns the number of edges (i.e. children) this node has.
    fn edges_count(&self) -> usize {
        self.edges.iter().filter(|o| o.is_some()).count()
    }

    /// Returns the position of this node in the parent node's `edges` array.
    fn parent_idx(&self) -> Option<usize> {
        self.parent_idx.map(|v| v as usize)
    }
}

/// Points to a node in the tree.
#[derive(Clone, Copy, Encode, Decode, PartialEq, Eq)]
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
    fn node(&self) -> u32 {
        self.node
    }
}

/// Points to a specific key/value pair within a node in the tree.
#[derive(Clone, Copy)]
pub(super) struct KVHandle {
    /// Index of the node in the `entries` storage.
    node: NodeHandle,
    /// Index of the key/value pair within the node. This is a pointer
    /// to the position in the `keys`/`vals` array.
    idx: u32,
}

impl KVHandle {
    /// Creates a new `KVHandle` from a `NodeHandle` and an `idx`.
    /// The `idx` is the node's index in the parent node's `edges` array.
    pub(super) fn new(node: NodeHandle, idx: usize) -> Self {
        Self { node, idx: idx as u32 }
    }

    /// Returns the `NodeHandle` for this node.
    /// The index should point to the nodes position in the `entries` storage.
    pub(super) fn node(&self) -> NodeHandle {
        self.node
    }

    /// Returns the position in the parent node's `edges` array to which
    /// this handle points.
    fn idx(self) -> usize {
        self.idx as usize
    }
}
