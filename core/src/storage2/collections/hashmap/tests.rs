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
use crate::{
    env,
    storage2::traits::{
        KeyPtr,
        SpreadLayout,
    },
};
use ink_primitives::Key;

#[cfg(feature = "ink-fuzz-tests")]
use itertools::Itertools;

/// Conducts repeated insert and remove operations into the map by iterating
/// over `xs`. For each odd `x` in `xs` a defined number of insert operations
/// (`inserts_each`) is executed. For each even `x` it's asserted that the
/// previously inserted elements are in the map and they are removed subsequently.
///
/// The reasoning behind this even/odd sequence is to introduce some
/// randomness into when elements are inserted/removed.
///
/// `inserts_each` was chosen as `u8` to keep the number of inserts per `x` in
/// a reasonable range.
#[cfg(feature = "ink-fuzz-tests")]
fn insert_and_remove(xs: Vec<i32>, inserts_each: u8) {
    let mut map = <StorageHashMap<i32, i32>>::new();
    let mut cnt_inserts = 0;
    let mut previous_even_x = None;
    let inserts_each = inserts_each as i32;

    xs.into_iter().for_each(|x| {
        if x % 2 == 0 {
            // On even numbers we insert
            for key in x..x + inserts_each {
                let val = key * 10;
                if let None = map.insert(key, val) {
                    assert_eq!(map.get(&key), Some(&val));
                    cnt_inserts += 1;
                }
                assert_eq!(map.len(), cnt_inserts);
            }
            if let None = previous_even_x {
                previous_even_x = Some(x);
            }
        } else if x % 2 == 1 && previous_even_x.is_some() {
            // If it's an odd number and we inserted in a previous run we assert
            // that the last insert worked correctly and remove the elements again.
            //
            // It can happen that after one insert run there are many more
            // insert runs (i.e. even `x` in `xs`) before we remove the numbers
            // of the last run again. This is intentional, as to include testing
            // if subsequent insert operations have an effect on already inserted
            // items.
            let x = previous_even_x.unwrap();
            for key in x..x + inserts_each {
                let val = key * 10;
                assert_eq!(map.get(&key), Some(&val));
                assert_eq!(map.take(&key), Some(val));
                assert_eq!(map.get(&key), None);
                cnt_inserts -= 1;
                assert_eq!(map.len(), cnt_inserts);
            }
            previous_even_x = None;
        }
    });
}

#[test]
fn new_works() {
    // `StorageHashMap::new`
    let hmap = <StorageHashMap<u8, i32>>::new();
    assert!(hmap.is_empty());
    assert_eq!(hmap.len(), 0);
    assert!(hmap.iter().next().is_none());
    // `StorageHashMap::default`
    let default = <StorageHashMap<u8, i32> as Default>::default();
    assert!(default.is_empty());
    assert_eq!(default.len(), 0);
    assert!(default.iter().next().is_none());
    // `StorageHashMap::new` and `StorageHashMap::default` should be equal.
    assert_eq!(hmap, default);
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

#[test]
fn defrag_works() {
    let expected = [(b'A', 1), (b'D', 4)]
        .iter()
        .copied()
        .collect::<StorageHashMap<u8, i32>>();
    // Defrag without limits:
    let mut hmap = [(b'A', 1), (b'B', 2), (b'C', 3), (b'D', 4)]
        .iter()
        .copied()
        .collect::<StorageHashMap<u8, i32>>();
    assert_eq!(hmap.defrag(None), 0);
    assert_eq!(hmap.take(&b'B'), Some(2));
    assert_eq!(hmap.take(&b'C'), Some(3));
    assert_eq!(hmap.defrag(None), 2);
    assert_eq!(hmap.defrag(None), 0);
    assert_eq!(hmap, expected);
    // Defrag with limits:
    let mut hmap = [(b'A', 1), (b'B', 2), (b'C', 3), (b'D', 4)]
        .iter()
        .copied()
        .collect::<StorageHashMap<u8, i32>>();
    assert_eq!(hmap.defrag(None), 0);
    assert_eq!(hmap.take(&b'B'), Some(2));
    assert_eq!(hmap.take(&b'C'), Some(3));
    assert_eq!(hmap.defrag(Some(1)), 1);
    assert_eq!(hmap.defrag(Some(1)), 1);
    assert_eq!(hmap.defrag(Some(1)), 0);
    assert_eq!(hmap, expected);
}

#[test]
fn spread_layout_push_pull_works() -> env::Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let hmap1 = [(b'A', 1), (b'B', 2), (b'C', 3), (b'D', 4)]
            .iter()
            .copied()
            .collect::<StorageHashMap<u8, i32>>();
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&hmap1, &mut KeyPtr::from(root_key));
        // Load the pushed storage vector into another instance and check that
        // both instances are equal:
        let hmap2 = <StorageHashMap<u8, i32> as SpreadLayout>::pull_spread(
            &mut KeyPtr::from(root_key),
        );
        assert_eq!(hmap1, hmap2);
        Ok(())
    })
}

#[test]
#[should_panic(expected = "storage entry was empty")]
fn spread_layout_clear_works() {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let hmap1 = [(b'A', 1), (b'B', 2), (b'C', 3), (b'D', 4)]
            .iter()
            .copied()
            .collect::<StorageHashMap<u8, i32>>();
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&hmap1, &mut KeyPtr::from(root_key));
        // It has already been asserted that a valid instance can be pulled
        // from contract storage after a push to the same storage region.
        //
        // Now clear the associated storage from `hmap1` and check whether
        // loading another instance from this storage will panic since the
        // vector's length property cannot read a value:
        SpreadLayout::clear_spread(&hmap1, &mut KeyPtr::from(root_key));
        let _ = <StorageHashMap<u8, i32> as SpreadLayout>::pull_spread(
            &mut KeyPtr::from(root_key),
        );
        Ok(())
    })
    .unwrap()
}

#[cfg(feature = "ink-fuzz-tests")]
#[quickcheck]
fn randomized_inserts_and_removes_hm(xs: Vec<i32>, inserts_each: u8) {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        insert_and_remove(xs, inserts_each);
        Ok(())
    })
    .unwrap()
}

/// Inserts all elements from `xs`. Then removes each `xth` element from the map
/// and asserts that all non-`xth` elements are still in the map.
#[cfg(feature = "ink-fuzz-tests")]
#[quickcheck]
fn randomized_removes(xs: Vec<i32>, xth: usize) {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let xs: Vec<i32> = xs.into_iter().unique().collect();
        let xth = xth.max(1);
        let mut map = <StorageHashMap<i32, i32>>::new();
        let mut len = map.len();

        // when
        // 1) insert all
        xs.iter().for_each(|i| {
            assert_eq!(map.insert(*i, i * 10), None);
            len += 1;
            assert_eq!(map.len(), len);
        });

        // 2) remove every `xth` element of `xs` from the map
        xs.iter().enumerate().for_each(|(x, i)| {
            if x % xth == 0 {
                assert_eq!(map.take(&i), Some(i * 10));
                len -= 1;
            }
            assert_eq!(map.len(), len);
        });

        // then
        // everything else must still be get-able
        xs.iter().enumerate().for_each(|(x, i)| {
            if x % xth != 0 {
                assert_eq!(map.get(&i), Some(&(i * 10)));
            }
        });

        Ok(())
    })
    .unwrap()
}
