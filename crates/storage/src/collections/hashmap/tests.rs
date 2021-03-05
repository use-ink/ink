// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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
    traits::{
        KeyPtr,
        SpreadLayout,
    },
    Lazy,
};
use ink_primitives::Key;

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

fn filled_hmap() -> StorageHashMap<u8, i32> {
    [(b'A', 1), (b'B', 2), (b'C', 3), (b'D', 4)]
        .iter()
        .copied()
        .collect::<StorageHashMap<u8, i32>>()
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
    let hmap = filled_hmap();
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
    let mut hmap = filled_hmap();
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
    let hmap = filled_hmap();
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
    let hmap = filled_hmap();
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
    let hmap = filled_hmap();
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
    let mut hmap = filled_hmap();
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
fn spread_layout_push_pull_works() -> ink_env::Result<()> {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let hmap1 = filled_hmap();
        push_hmap(&hmap1);
        // Load the pushed storage hmap into another instance and check that
        // both instances are equal:
        let hmap2 = pull_hmap();
        assert_eq!(hmap1, hmap2);
        Ok(())
    })
}

#[test]
#[should_panic(expected = "storage entry was empty")]
fn spread_layout_clear_works() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let hmap1 = filled_hmap();
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&hmap1, &mut KeyPtr::from(root_key));
        // It has already been asserted that a valid instance can be pulled
        // from contract storage after a push to the same storage region.
        //
        // Now clear the associated storage from `hmap1` and check whether
        // loading another instance from this storage will panic since the
        // hmap's length property cannot read a value:
        SpreadLayout::clear_spread(&hmap1, &mut KeyPtr::from(root_key));
        let _ = <StorageHashMap<u8, i32> as SpreadLayout>::pull_spread(
            &mut KeyPtr::from(root_key),
        );
        Ok(())
    })
    .unwrap()
}

#[test]
fn storage_is_cleared_completely_after_pull_lazy() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        // given
        let root_key = Key::from([0x42; 32]);
        let lazy_hmap = Lazy::new(filled_hmap());
        SpreadLayout::push_spread(&lazy_hmap, &mut KeyPtr::from(root_key));
        let pulled_hmap = <Lazy<StorageHashMap<u8, i32>> as SpreadLayout>::pull_spread(
            &mut KeyPtr::from(root_key),
        );

        // when
        SpreadLayout::clear_spread(&pulled_hmap, &mut KeyPtr::from(root_key));

        // then
        let contract_id = ink_env::test::get_current_contract_account_id::<
            ink_env::DefaultEnvironment,
        >()
        .expect("Cannot get contract id");
        let used_cells = ink_env::test::count_used_storage_cells::<
            ink_env::DefaultEnvironment,
        >(&contract_id)
        .expect("used cells must be returned");
        assert_eq!(used_cells, 0);

        Ok(())
    })
    .unwrap()
}

#[test]
#[should_panic(expected = "storage entry was empty")]
fn drop_works() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let root_key = Key::from([0x42; 32]);

        // if the setup panics it should not cause the test to pass
        let setup_result = std::panic::catch_unwind(|| {
            let hmap = filled_hmap();
            SpreadLayout::push_spread(&hmap, &mut KeyPtr::from(root_key));
            let _ = <StorageHashMap<u8, i32> as SpreadLayout>::pull_spread(
                &mut KeyPtr::from(root_key),
            );
            // hmap is dropped which should clear the cells
        });
        assert!(setup_result.is_ok(), "setup should not panic");

        let contract_id = ink_env::test::get_current_contract_account_id::<
            ink_env::DefaultEnvironment,
        >()
        .expect("Cannot get contract id");
        let used_cells = ink_env::test::count_used_storage_cells::<
            ink_env::DefaultEnvironment,
        >(&contract_id)
        .expect("used cells must be returned");
        assert_eq!(used_cells, 0);

        let _ = <StorageHashMap<u8, i32> as SpreadLayout>::pull_spread(
            &mut KeyPtr::from(root_key),
        );
        Ok(())
    })
    .unwrap()
}
