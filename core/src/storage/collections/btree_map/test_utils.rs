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

use crate::storage::{
    alloc::{
        AllocateUsing,
        BumpAlloc,
        Initialize,
    },
    collections::btree_map::node::NodeHandle,
    BTreeMap,
};
use ink_primitives::Key;
use itertools::Itertools;

/// Creates an empty map.
pub fn empty_map() -> BTreeMap<i32, i32> {
    unsafe {
        let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
        BTreeMap::allocate_using(&mut alloc).initialize_into(())
    }
}

/// Creates a map pre-filled with some key/value pairs.
pub fn filled_map() -> BTreeMap<i32, i32> {
    let mut map = empty_map();
    map.insert(5, 50);
    map.insert(42, 420);
    map.insert(1337, 13370);
    map.insert(77, 770);
    assert_eq!(map.len(), 4);
    map
}

/// Returns all edges in the tree as one Vec.
pub fn all_edges(map: &BTreeMap<i32, i32>) -> Vec<u32> {
    let mut v = Vec::new();
    let mut processed_nodes = 0;
    let mut node_index = 0;
    loop {
        if processed_nodes == map.node_count() {
            break
        }

        // We iterate over all storage entities of the tree and skip vacant entities.
        let handle = NodeHandle::new(node_index);
        if let Some(node) = map.get_node(handle) {
            let edges = node
                .edges()
                .to_vec()
                .into_iter()
                .filter_map(|x| x.map(|v| v.node()));
            v.extend(edges);
            processed_nodes += 1;
        }
        node_index += 1;
    }
    v
}

/// Returns `true` if every edge exists only once in the tree.
/// If duplicate edges are found each duplicate is printed to the console.
pub fn every_edge_exists_only_once(map: &BTreeMap<i32, i32>) -> bool {
    let all_edges = all_edges(map);
    let unique_edges: Vec<u32> = all_edges.clone().into_iter().unique().collect();

    let only_unique_edges = all_edges.len() == unique_edges.len();
    if !only_unique_edges {
        unique_edges.iter().for_each(|x| {
            if all_edges.iter().any(|a| *a == *x) {
                eprintln!("duplicate {:?}", x);
            }
        });
    }
    only_unique_edges
}

/// Conducts repeated insert and remove operations into the map by iterating
/// over `xs`. For each odd number a defined number of insert operations
/// are executed. For each even number it's asserted that the previously
/// inserted elements are in the map and they are removed subsequently.
///
/// Using this scheme we get a sequence of insert and remove operations.
pub fn insert_and_remove(xs: Vec<i32>) {
    let mut map = empty_map();
    let mut count_inserts = 0;
    let mut previous_even_x = None;
    let number_inserts = 3;

    xs.iter().for_each(|x| {
        let x = *x;
        if x % 2 == 0 {
            // On even numbers we insert new nodes.
            for a in x..x + number_inserts {
                if let None = map.insert(a, a * 10) {
                    count_inserts += 1;
                }
                assert_eq!(map.len(), count_inserts);
            }
            previous_even_x = Some(x);
        } else if x % 2 == 1 && previous_even_x.is_some() {
            // if it's an odd number and we inserted in the previous run we assert
            // that the insert worked correctly and remove the elements again.
            let x = previous_even_x.unwrap();
            for a in x..x + number_inserts {
                assert_eq!(map.get(&a), Some(&(a * 10)));
                assert_eq!(map.remove(&a), Some(a * 10));
                assert_eq!(map.get(&a), None);
                count_inserts -= 1;
                assert_eq!(map.len(), count_inserts);
            }
            previous_even_x = None;
        }
        assert!(every_edge_exists_only_once(&map));
    });
}

/// Asserts that there is no node in storage for the range `0..max_node_count`.
pub fn storage_empty(map: &BTreeMap<i32, i32>, max_node_count: u32) -> bool {
    for i in 0..max_node_count {
        assert!(map.get_node(NodeHandle::new(i)).is_none());
    }
    true
}
