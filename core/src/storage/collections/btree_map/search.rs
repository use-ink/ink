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

use self::SearchResult::{
    Found,
    NotFound,
};
use crate::storage::btree_map::{
    impls::{
        BTreeMap,
        HandleType::{
            Branch,
            Leaf,
        },
        CAPACITY,
    },
    node::{
        KVHandle,
        Node,
        NodeHandle,
    },
};
use core::{
    borrow::Borrow,
    cmp::{
        Ord,
        Ordering,
    },
};
use scale::Codec;

/// Result of a tree search.
pub(super) enum SearchResult {
    /// Found the entry at the supplied position.
    Found(KVHandle),
    /// No search result, contains the position where an insert could be made.
    NotFound(KVHandle),
}

/// Searches the tree for `key`.
///
/// If found returns `Found(pos)`
/// If not found returns `NotFound(last_pos_searched)`.
pub(super) fn search_tree<K, V, Q>(tree: &BTreeMap<K, V>, key: &Q) -> SearchResult
where
    Q: Ord,
    K: Ord + Borrow<Q> + Codec,
    V: Codec,
{
    let current_root = tree.root();
    if tree.is_empty() || current_root.is_none() {
        return NotFound(KVHandle::new(NodeHandle::new(0), 0))
    }

    let mut cur =
        current_root.expect("we would already have returned if no root exists; qed");
    loop {
        let node = tree.get_node(cur).expect(
            "node which is iterated over is either root or child node, \
             but it always exists; qed",
        );
        match search_node(&node, tree.keys_in_node(cur), cur, key) {
            Found(handle) => return Found(handle),
            NotFound(handle) => {
                match tree.get_handle_type(handle.node()) {
                    Leaf => return NotFound(handle),
                    Branch => {
                        // Go down then
                        cur = tree
                            .descend(handle)
                            .expect("a branch node always has a child; qed");
                        continue
                    }
                }
            }
        }
    }
}

/// Conducts a linear search for `key` in the elements contained in `node`.
///
/// If found returns `Found(pos)`
/// If not found returns `NotFound(last_pos_searched)`.
pub(super) fn search_node<K, V, Q>(
    node: &Node<K, V>,
    keys_in_node: [Option<&K>; CAPACITY],
    node_handle: NodeHandle,
    key: &Q,
) -> SearchResult
where
    Q: Ord,
    K: Borrow<Q>,
{
    let iter = keys_in_node.iter().enumerate();
    for (i, k) in iter {
        match k {
            None => return NotFound(KVHandle::new(node_handle, i)),
            Some(node_key) => {
                match key.cmp((**node_key).borrow()) {
                    Ordering::Greater => {}
                    Ordering::Equal => return Found(KVHandle::new(node_handle, i)),
                    Ordering::Less => return NotFound(KVHandle::new(node_handle, i)),
                }
            }
        }
    }
    NotFound(KVHandle::new(node_handle, node.len()))
}
