// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

use super::HashMap as StorageHashMap;

#[test]
fn new_vec_works() {
    // `StorageHashMap::new`
    let vec = <StorageHashMap<u8, i32>>::new();
    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);
    assert!(vec.iter().next().is_none());
    // `StorageHashMap::default`
    let default = <StorageHashMap<u8, i32> as Default>::default();
    assert!(default.is_empty());
    assert_eq!(default.len(), 0);
    assert!(default.iter().next().is_none());
    // `StorageHashMap::new` and `StorageHashMap::default` should be equal.
    assert_eq!(vec, default);
}

#[test]
fn from_iterator_works() {
    let test_values = [(b'A', 1), (b'B', 2), (b'C', 3), (b'D', 4)];
    let hmap = test_values
        .iter()
        .copied()
        .collect::<StorageHashMap<u8, i32>>();
    assert!(!hmap.is_empty());
    assert_eq!(hmap.len(), 4);
    assert_eq!(hmap, {
        let mut hmap = <StorageHashMap<u8, i32>>::new();
        for (key, value) in &test_values {
            assert_eq!(hmap.insert(*key, *value), None);
        }
        hmap
    });
}

#[test]
fn from_empty_iterator_works() {
    assert_eq!(
        [].iter().copied().collect::<StorageHashMap<u8, i32>>(),
        <StorageHashMap<u8, i32>>::new(),
    );
}

#[test]
fn contains_key_works() {
    // Empty hash map.
    let hmap = <StorageHashMap<u8, i32>>::new();
    assert!(!hmap.contains_key(&b'A'));
    assert!(!hmap.contains_key(&b'E'));
    // Filled hash map.
    let hmap = [(b'A', 1), (b'B', 2), (b'C', 3), (b'D', 4)]
        .iter()
        .copied()
        .collect::<StorageHashMap<u8, i32>>();
    assert!(hmap.contains_key(&b'A'));
    assert!(hmap.contains_key(&b'B'));
    assert!(hmap.contains_key(&b'C'));
    assert!(hmap.contains_key(&b'D'));
    assert!(!hmap.contains_key(&b'E'));
}

#[test]
fn get_works() {
    // Empty hash map.
    let hmap = <StorageHashMap<u8, i32>>::new();
    assert_eq!(hmap.get(&b'A'), None);
    assert_eq!(hmap.get(&b'E'), None);
    // Filled hash map: `get`
    let hmap = [(b'A', 1), (b'B', 2), (b'C', 3), (b'D', 4)]
        .iter()
        .copied()
        .collect::<StorageHashMap<u8, i32>>();
    assert_eq!(hmap.get(&b'A'), Some(&1));
    assert_eq!(hmap.get(&b'B'), Some(&2));
    assert_eq!(hmap.get(&b'C'), Some(&3));
    assert_eq!(hmap.get(&b'D'), Some(&4));
    assert_eq!(hmap.get(&b'E'), None);
    // Filled hash map: `get_mut`
    let mut hmap = hmap;
    assert_eq!(hmap.get_mut(&b'A'), Some(&mut 1));
    assert_eq!(hmap.get_mut(&b'B'), Some(&mut 2));
    assert_eq!(hmap.get_mut(&b'C'), Some(&mut 3));
    assert_eq!(hmap.get_mut(&b'D'), Some(&mut 4));
    assert_eq!(hmap.get_mut(&b'E'), None);
}

#[test]
fn insert_works() {
    let mut hmap = <StorageHashMap<u8, i32>>::new();
    // Start with an empty hash map.
    assert_eq!(hmap.len(), 0);
    assert_eq!(hmap.get(&b'A'), None);
    // Insert first value.
    hmap.insert(b'A', 1);
    assert_eq!(hmap.len(), 1);
    assert_eq!(hmap.get(&b'A'), Some(&1));
    assert_eq!(hmap.get_mut(&b'A'), Some(&mut 1));
    // Update the inserted value.
    hmap.insert(b'A', 2);
    assert_eq!(hmap.len(), 1);
    assert_eq!(hmap.get(&b'A'), Some(&2));
    assert_eq!(hmap.get_mut(&b'A'), Some(&mut 2));
    // Insert another value.
    hmap.insert(b'B', 3);
    assert_eq!(hmap.len(), 2);
    assert_eq!(hmap.get(&b'B'), Some(&3));
    assert_eq!(hmap.get_mut(&b'B'), Some(&mut 3));
}

#[test]
fn take_works() {
    // Empty hash map.
    let mut hmap = <StorageHashMap<u8, i32>>::new();
    assert_eq!(hmap.take(&b'A'), None);
    assert_eq!(hmap.take(&b'E'), None);
    // Filled hash map: `get`
    let mut hmap = [(b'A', 1), (b'B', 2), (b'C', 3), (b'D', 4)]
        .iter()
        .copied()
        .collect::<StorageHashMap<u8, i32>>();
    assert_eq!(hmap.len(), 4);
    assert_eq!(hmap.take(&b'A'), Some(1));
    assert_eq!(hmap.len(), 3);
    assert_eq!(hmap.take(&b'A'), None);
    assert_eq!(hmap.len(), 3);
    assert_eq!(hmap.take(&b'B'), Some(2));
    assert_eq!(hmap.len(), 2);
    assert_eq!(hmap.take(&b'C'), Some(3));
    assert_eq!(hmap.len(), 1);
    assert_eq!(hmap.take(&b'D'), Some(4));
    assert_eq!(hmap.len(), 0);
    assert_eq!(hmap.take(&b'E'), None);
    assert_eq!(hmap.len(), 0);
}

#[test]
fn iter_next_works() {
    let hmap = [(b'A', 1), (b'B', 2), (b'C', 3), (b'D', 4)]
        .iter()
        .copied()
        .collect::<StorageHashMap<u8, i32>>();
    // Test iterator over shared references:
    let mut iter = hmap.iter();
    assert_eq!(iter.count(), 4);
    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.next(), Some((&b'A', &1)));
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.next(), Some((&b'B', &2)));
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.count(), 2);
    assert_eq!(iter.next(), Some((&b'C', &3)));
    assert_eq!(iter.size_hint(), (1, Some(1)));
    assert_eq!(iter.next(), Some((&b'D', &4)));
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.count(), 0);
    assert_eq!(iter.next(), None);
    // Test iterator over exclusive references:
    let mut hmap = hmap;
    let mut iter = hmap.iter_mut();
    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.next(), Some((&b'A', &mut 1)));
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.next(), Some((&b'B', &mut 2)));
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.next(), Some((&b'C', &mut 3)));
    assert_eq!(iter.size_hint(), (1, Some(1)));
    assert_eq!(iter.next(), Some((&b'D', &mut 4)));
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.count(), 0);
}

#[test]
fn values_next_works() {
    let hmap = [(b'A', 1), (b'B', 2), (b'C', 3), (b'D', 4)]
        .iter()
        .copied()
        .collect::<StorageHashMap<u8, i32>>();
    // Test iterator over shared references:
    let mut iter = hmap.values();
    assert_eq!(iter.count(), 4);
    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.next(), Some(&1));
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.count(), 2);
    assert_eq!(iter.next(), Some(&3));
    assert_eq!(iter.size_hint(), (1, Some(1)));
    assert_eq!(iter.next(), Some(&4));
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.count(), 0);
    assert_eq!(iter.next(), None);
    // Test iterator over exclusive references:
    let mut hmap = hmap;
    let mut iter = hmap.values_mut();
    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.next(), Some(&mut 1));
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.next(), Some(&mut 2));
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.next(), Some(&mut 3));
    assert_eq!(iter.size_hint(), (1, Some(1)));
    assert_eq!(iter.next(), Some(&mut 4));
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.count(), 0);
}

#[test]
fn keys_next_works() {
    let hmap = [(b'A', 1), (b'B', 2), (b'C', 3), (b'D', 4)]
        .iter()
        .copied()
        .collect::<StorageHashMap<u8, i32>>();
    let mut iter = hmap.keys();
    assert_eq!(iter.count(), 4);
    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.next(), Some(&b'A'));
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.next(), Some(&b'B'));
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.count(), 2);
    assert_eq!(iter.next(), Some(&b'C'));
    assert_eq!(iter.size_hint(), (1, Some(1)));
    assert_eq!(iter.next(), Some(&b'D'));
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.count(), 0);
    assert_eq!(iter.next(), None);
}
