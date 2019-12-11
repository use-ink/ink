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

use core::{
    cmp::{Ord, Ordering},
    borrow::Borrow,
};
use scale::Codec;
use crate::storage::btree_map::impls::{HandleType, KVHandle, Node, BTreeMap};

/// ToDo
pub enum SearchResult {
    /// Found the to-be-searched entry at the supplied position.
    Found(KVHandle),
    /// No search result, contains the position where an insert could be made.
    NotFound(KVHandle),
}

/// Searches the tree for `key`.
pub(crate) fn search_tree<K, V, Q>(
    tree: &BTreeMap<K, V>,
    key: &Q
) -> SearchResult
where
    Q: Ord,
    K: Ord + Borrow<Q> + Codec,
    V: Codec,
{
    let current_root = tree.root();
    if tree.len() == 0 || current_root.is_none() {
        return SearchResult::NotFound(KVHandle::new(0, 0))
    }

    let mut cur = current_root.expect("46");
    loop {
        let node = tree.get_node(&cur.into())
            .expect(
                "node which is iterated over is either root or child node, \
                but always exists; qed"
            );
        match search_node(&node, cur, key) {
            SearchResult::Found(handle) => return SearchResult::Found(handle),
            SearchResult::NotFound(handle) => {
                match tree.get_handle_type(&handle.into()) {
                    HandleType::Leaf => return SearchResult::NotFound(handle),
                    HandleType::Internal => {
                        // Go down then
                        cur = tree
                            .descend(&handle)
                            .expect("an internal node always has a child; qed")
                            .node();
                        continue;
                    }
                }
            }
        }
    }
}

/// Searches for `key` in the elements contained in a particular node .
fn search_node<K, V, Q>(
    node: &Node<K, V>,
    node_index: u32,
    key: &Q,
) -> SearchResult
where
    Q: Ord,
    K: Borrow<Q>,
{
    match search_linear(node, key) {
        (idx, true) => SearchResult::Found(
            KVHandle::new(node_index, idx)
        ),
        (idx, false) => SearchResult::NotFound(
            KVHandle::new(node_index, idx)
        )
    }
}

/// Conducts a linear search for `key` in the elements contained in a node.
pub fn search_linear<K, V, Q>(
    node: &Node<K, V>,
    key: &Q
) -> (u32, bool)
where
    Q: Ord,
    K: Borrow<Q>,
{
    let iter = node.keys.iter().enumerate();
    for (i, k) in iter {
        match k {
            None => return (i as u32, false),
            Some(node_key) => {
                match key.cmp(node_key.borrow()) {
                    Ordering::Greater => {},
                    Ordering::Equal => return (i as u32, true),
                    Ordering::Less => return (i as u32, false)
                }
            }
        }
    }
    // ToDo maybe return KVHandle instead of u32
    (node.len, false)
}
