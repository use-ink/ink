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

use super::{
    Entry,
    Entry::{
        Occupied,
        Vacant,
    },
    HashMap as StorageHashMap,
};

use crate::{
    env,
    storage2::traits::{
        KeyPtr,
        SpreadLayout,
    },
};
use ink_primitives::Key;
use num_traits::ToPrimitive;

/// Returns a prefilled `HashMap` with `[('A', 13), ['B', 23])`.
fn prefilled_hmap() -> StorageHashMap<u8, i32> {
    let test_values = [(b'A', 13), (b'B', 23)];
    test_values
        .iter()
        .copied()
        .collect::<StorageHashMap<u8, i32>>()
}

/// Returns always the same `KeyPtr`.
fn key_ptr() -> KeyPtr {
    let root_key = Key::from([0x42; 32]);
    KeyPtr::from(root_key)
}

/// Pushes a `HashMap` instance into the contract storage.
fn push_hmap(hmap: &StorageHashMap<u8, i32>) {
    SpreadLayout::push_spread(hmap, &mut key_ptr());
}

/// Pulls a `HashMap` instance from the contract storage.
fn pull_hmap() -> StorageHashMap<u8, i32> {
    <StorageHashMap<u8, i32> as SpreadLayout>::pull_spread(&mut key_ptr())
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
        push_hmap(&hmap1);
        // Load the pushed storage vector into another instance and check that
        // both instances are equal:
        let hmap2 = pull_hmap();
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

#[test]
fn entry_api_insert_inexistent_works_with_empty() {
    // given
    let mut hmap = <StorageHashMap<u8, bool>>::new();
    assert!(matches!(hmap.entry(b'A'), Vacant(_)));
    assert!(hmap.get(&b'A').is_none());

    // when
    assert!(*hmap.entry(b'A').or_insert(true));

    // then
    assert_eq!(hmap.get(&b'A'), Some(&true));
    assert_eq!(hmap.len(), 1);
}

#[test]
fn entry_api_insert_existent_works() {
    // given
    let mut hmap = prefilled_hmap();
    match hmap.entry(b'A') {
        Vacant(_) => panic!(),
        Occupied(o) => assert_eq!(o.get(), &13),
    }

    // when
    hmap.entry(b'A').or_insert(77);

    // then
    assert_eq!(hmap.get(&b'A'), Some(&13));
    assert_eq!(hmap.len(), 2);
}

#[test]
fn entry_api_mutations_work_with_push_pull() -> env::Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let hmap1 = prefilled_hmap();
        assert_eq!(hmap1.get(&b'A'), Some(&13));
        push_hmap(&hmap1);

        let mut hmap2 = pull_hmap();
        assert_eq!(hmap2.get(&b'A'), Some(&13));

        // when
        let v = hmap2.entry(b'A').or_insert(42);
        *v += 1;
        assert_eq!(hmap2.get(&b'A'), Some(&14));
        push_hmap(&hmap2);

        // then
        let hmap3 = pull_hmap();
        assert_eq!(hmap3.get(&b'A'), Some(&14));
        Ok(())
    })
}

#[test]
fn entry_api_simple_insert_with_works() {
    // given
    let mut hmap = prefilled_hmap();

    // when
    assert!(hmap.get(&b'C').is_none());
    let v = hmap.entry(b'C').or_insert_with(|| 42);

    // then
    assert_eq!(*v, 42);
    assert_eq!(hmap.get(&b'C'), Some(&42));
    assert_eq!(hmap.len(), 3);
}

#[test]
fn entry_api_simple_default_insert_works() {
    // given
    let mut hmap = <StorageHashMap<u8, bool>>::new();

    // when
    let v = hmap.entry(b'A').or_default();

    // then
    assert_eq!(*v, false);
    assert_eq!(hmap.get(&b'A'), Some(&false));
}

#[test]
fn entry_api_insert_with_works_with_mutations() {
    // given
    let mut hmap = <StorageHashMap<u8, i32>>::new();
    let v = hmap.entry(b'A').or_insert_with(|| 42);
    assert_eq!(*v, 42);

    // when
    *v += 1;

    // then
    assert_eq!(hmap.get(&b'A'), Some(&43));
    assert_eq!(hmap.len(), 1);
}

#[test]
fn entry_api_insert_with_works_with_push_pull() -> env::Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut hmap1 = <StorageHashMap<u8, i32>>::new();
        let value = hmap1.entry(b'A').or_insert_with(|| 42);

        // when
        *value = 43;
        push_hmap(&hmap1);

        // then
        let hmap2 = pull_hmap();
        assert_eq!(hmap2.get(&b'A'), Some(&43));
        Ok(())
    })
}

#[test]
fn entry_api_simple_insert_with_key_works() {
    // given
    let mut hmap = <StorageHashMap<u8, i32>>::new();

    // when
    let _ = hmap
        .entry(b'A')
        .or_insert_with_key(|key| key.to_i32().unwrap() * 2);

    // then
    assert_eq!(hmap.get(&b'A'), Some(&130));
}

#[test]
fn entry_api_key_get_works_with_nonexistent() {
    let mut hmap = <StorageHashMap<u8, i32>>::new();
    assert_eq!(hmap.entry(b'A').key(), &b'A');
}

#[test]
fn entry_api_key_get_works_with_existent() {
    let mut hmap = prefilled_hmap();
    assert_eq!(hmap.entry(b'A').key(), &b'A');
    assert_eq!(hmap.entry(b'B').key(), &b'B');
}

#[test]
fn entry_api_and_modify_has_no_effect_for_nonexistent() {
    // given
    let mut hmap = <StorageHashMap<u8, i32>>::new();

    // when
    hmap.entry(b'B').and_modify(|e| *e += 1).or_insert(42);

    // then
    assert_eq!(hmap.get(&b'B'), Some(&42));
}

#[test]
fn entry_api_and_modify_works_for_existent() {
    // given
    let mut hmap = prefilled_hmap();

    // when
    assert_eq!(hmap.get(&b'B'), Some(&23));
    hmap.entry(b'B').and_modify(|e| *e += 1).or_insert(7);

    // then
    assert_eq!(hmap.get(&b'B'), Some(&24));
}

#[test]
fn entry_api_occupied_entry_api_works_with_push_pull() -> env::Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut hmap1 = prefilled_hmap();
        assert_eq!(hmap1.get(&b'A'), Some(&13));
        match hmap1.entry(b'A') {
            Entry::Occupied(mut o) => {
                assert_eq!(o.key(), &b'A');
                assert_eq!(o.insert(15), 13);
            }
            Entry::Vacant(_) => panic!(),
        }
        push_hmap(&hmap1);

        // when
        let mut hmap2 = pull_hmap();
        assert_eq!(hmap2.get(&b'A'), Some(&15));
        match hmap2.entry(b'A') {
            Entry::Occupied(o) => {
                assert_eq!(o.remove_entry(), (b'A', 15));
            }
            Entry::Vacant(_) => panic!(),
        }
        push_hmap(&hmap2);

        // then
        let hmap3 = pull_hmap();
        assert_eq!(hmap3.get(&b'A'), None);

        Ok(())
    })
}

#[test]
fn entry_api_vacant_api_works() {
    let mut hmap = <StorageHashMap<u8, i32>>::new();
    match hmap.entry(b'A') {
        Entry::Occupied(_) => panic!(),
        Entry::Vacant(v) => {
            assert_eq!(v.key(), &b'A');
            assert_eq!(v.into_key(), b'A');
        }
    }
}

#[test]
fn entry_api_vacant_api_works_with_push_pull() -> env::Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut hmap1 = <StorageHashMap<u8, i32>>::new();
        match hmap1.entry(b'A') {
            Entry::Occupied(_) => panic!(),
            Entry::Vacant(v) => {
                let val = v.insert(42);
                *val += 1;
            }
        }
        push_hmap(&hmap1);

        // when
        let hmap2 = pull_hmap();

        // then
        assert_eq!(hmap2.get(&b'A'), Some(&43));
        Ok(())
    })
}
