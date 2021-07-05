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

use super::Stash as StorageStash;
use crate::{
    traits::{
        KeyPtr,
        SpreadLayout,
    },
    Lazy,
};
use ink_primitives::Key;

#[test]
fn regression_stash_unreachable_minified() {
    // This regression has been discovered in the ERC-721 example implementation
    // `approved_for_all_works` unit test. The fix was to adjust
    // `Stash::remove_vacant_entry` to update `header.last_vacant` if the
    // removed index was the last remaining vacant index in the stash.
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let mut stash: StorageStash<u32> = StorageStash::new();
        stash.put(1);
        stash.put(2);
        stash.take(0);
        stash.put(99);
        stash.take(1);
        stash.put(99);
        Ok(())
    })
    .unwrap()
}

#[test]
fn new_works() {
    // `StorageVec::new`
    let stash = <StorageStash<i32>>::new();
    assert!(stash.is_empty());
    assert_eq!(stash.len(), 0);
    assert_eq!(stash.get(0), None);
    assert!(stash.iter().next().is_none());
    // `StorageVec::default`
    let default = <StorageStash<i32> as Default>::default();
    assert!(default.is_empty());
    assert_eq!(default.len(), 0);
    assert_eq!(stash.get(0), None);
    assert!(default.iter().next().is_none());
    // `StorageVec::new` and `StorageVec::default` should be equal.
    assert_eq!(stash, default);
}

#[test]
fn from_iterator_works() {
    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    let stash = test_values.iter().copied().collect::<StorageStash<_>>();
    assert_eq!(stash, {
        let mut stash = StorageStash::new();
        for (index, value) in test_values.iter().enumerate() {
            assert_eq!(index as u32, stash.put(*value));
        }
        stash
    });
    assert_eq!(stash.len(), test_values.len() as u32);
    assert_eq!(stash.is_empty(), false);
}

#[test]
fn from_empty_iterator_works() {
    assert_eq!(
        [].iter().copied().collect::<StorageStash<i32>>(),
        StorageStash::new(),
    );
}

#[test]
fn take_from_filled_works() {
    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    let mut stash = test_values.iter().copied().collect::<StorageStash<_>>();
    for (index, expected_value) in test_values.iter().enumerate() {
        assert_eq!(stash.take(index as u32), Some(*expected_value));
    }
}

#[test]
fn take_from_empty_works() {
    let mut stash = <StorageStash<u8>>::new();
    assert_eq!(stash.take(0), None);
}

#[test]
fn take_out_of_bounds_works() {
    let mut stash = [b'A', b'B', b'C']
        .iter()
        .copied()
        .collect::<StorageStash<_>>();
    assert_eq!(stash.take(3), None);
}

#[test]
fn remove_from_filled_works() {
    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    let mut stash = test_values.iter().copied().collect::<StorageStash<_>>();

    let mut count = stash.len();
    for (index, val) in test_values.iter().enumerate() {
        let index = index as u32;
        assert_eq!(stash.get(index), Some(val));
        assert_eq!(unsafe { stash.remove_occupied(index) }, Some(()));
        assert_eq!(stash.get(index), None);
        count -= 1;
        assert_eq!(stash.len(), count);
    }
    assert_eq!(stash.len(), 0);
}

#[test]
fn remove_from_empty_works() {
    let mut stash = <StorageStash<u8>>::new();
    assert_eq!(unsafe { stash.remove_occupied(0) }, None);
}

#[test]
fn remove_out_of_bounds_works() {
    let mut stash = [b'A', b'B', b'C']
        .iter()
        .copied()
        .collect::<StorageStash<_>>();
    assert_eq!(unsafe { stash.remove_occupied(3) }, None);
}

#[test]
fn remove_works_with_spread_layout_push_pull() -> ink_env::Result<()> {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        // First populate some storage Stash and writes that to the contract storage using pull_spread
        // and some known Key.
        let stash = [b'A', b'B', b'C']
            .iter()
            .copied()
            .collect::<StorageStash<_>>();
        let root_key = Key::from([0x00; 32]);
        SpreadLayout::push_spread(&stash, &mut KeyPtr::from(root_key));

        // Then load another instance from the same key lazily and remove some of
        // the known-to-be-populated entries from it. Afterwards push_spread this second instance and
        // load yet another using pull_spread again.
        let mut stash2 =
            <StorageStash<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(unsafe { stash2.remove_occupied(0) }, Some(()));
        SpreadLayout::push_spread(&stash2, &mut KeyPtr::from(root_key));

        // This time we check from the third instance using
        // get if the expected cells are still there or have been successfully removed.
        let stash3 =
            <StorageStash<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(stash3.get(0), None);
        assert_eq!(stash3.get(1), Some(&b'B'));
        assert_eq!(stash3.get(2), Some(&b'C'));
        assert_eq!(stash3.len(), 2);

        Ok(())
    })
}

#[test]
fn get_works() {
    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    let mut stash = test_values.iter().copied().collect::<StorageStash<_>>();
    for (index, &expected_value) in test_values.iter().enumerate() {
        let mut expected_value = expected_value;
        let index = index as u32;
        assert_eq!(stash.get(index), Some(&expected_value));
        assert_eq!(stash.get_mut(index), Some(&mut expected_value));
        assert_eq!(&stash[index], &expected_value);
        assert_eq!(&mut stash[index], &mut expected_value);
    }
    // Get out of bounds works:
    let len = stash.len();
    assert_eq!(stash.get(len), None);
    assert_eq!(stash.get_mut(len), None);
    // Get vacant entry works:
    assert_eq!(stash.get(1), Some(&b'B'));
    assert_eq!(stash.get_mut(1), Some(&mut b'B'));
    assert_eq!(stash.take(1), Some(b'B'));
    assert_eq!(stash.get(1), None);
    assert_eq!(stash.get_mut(1), None);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "index out of bounds: the len is 3 but the index is 3")]
fn index_out_of_bounds_works() {
    let test_values = [b'a', b'b', b'c'];
    let stash = test_values.iter().copied().collect::<StorageStash<_>>();
    let _ = &stash[test_values.len() as u32];
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "index out of bounds: the len is 3 but the index is 3")]
fn index_mut_out_of_bounds_works() {
    let test_values = [b'a', b'b', b'c'];
    let mut stash = test_values.iter().copied().collect::<StorageStash<_>>();
    let _ = &mut stash[test_values.len() as u32];
}

#[test]
#[should_panic(expected = "indexed vacant entry: at index 1")]
fn index_vacant_works() {
    let test_values = [b'a', b'b', b'c'];
    let mut stash = test_values.iter().copied().collect::<StorageStash<_>>();
    assert_eq!(stash.take(1), Some(b'b'));
    let _ = &stash[1];
}

#[test]
#[should_panic(expected = "indexed vacant entry: at index 1")]
fn index_mut_vacant_works() {
    let test_values = [b'a', b'b', b'c'];
    let mut stash = test_values.iter().copied().collect::<StorageStash<_>>();
    assert_eq!(stash.take(1), Some(b'b'));
    let _ = &mut stash[1];
}

#[test]
fn len_is_empty_works() {
    let mut stash = StorageStash::new();
    assert_eq!(stash.len(), 0);
    assert!(stash.is_empty());
    stash.put(b'A');
    assert_eq!(stash.len(), 1);
    assert!(!stash.is_empty());
    stash.take(0);
    assert_eq!(stash.len(), 0);
    assert!(stash.is_empty());
}

#[test]
fn iter_works() {
    let stash = [b'A', b'B', b'C']
        .iter()
        .copied()
        .collect::<StorageStash<_>>();
    // Test iterator over shared references.
    let mut iter = stash.iter();
    assert_eq!(iter.count(), 3);
    assert_eq!(iter.next(), Some(&b'A'));
    assert_eq!(iter.count(), 2);
    assert_eq!(iter.next(), Some(&b'B'));
    assert_eq!(iter.count(), 1);
    assert_eq!(iter.next(), Some(&b'C'));
    assert_eq!(iter.count(), 0);
    assert_eq!(iter.next(), None);
    // Test iterator over exclusive references.
    let mut stash = stash;
    let mut iter = stash.iter_mut();
    assert_eq!(iter.next(), Some(&mut b'A'));
    assert_eq!(iter.next(), Some(&mut b'B'));
    assert_eq!(iter.next(), Some(&mut b'C'));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.count(), 0);
}

/// Create a stash that only has vacant entries.
fn create_vacant_stash() -> StorageStash<u8> {
    let mut stash = [b'A', b'B', b'C']
        .iter()
        .copied()
        .collect::<StorageStash<_>>();
    for i in 0..stash.len() {
        stash.take(i);
    }
    assert_eq!(stash.len(), 0);
    assert!(stash.is_empty());
    assert_eq!(stash.len_entries(), 3);
    stash
}

/// Create a stash where every second entry is vacant.
fn create_holey_stash() -> StorageStash<u8> {
    let elements = [b'A', b'B', b'C', b'D', b'E', b'F'];
    let mut stash = elements.iter().copied().collect::<StorageStash<_>>();
    for i in 0..stash.len() {
        stash.take(i * 2);
    }
    assert_eq!(stash.len() as usize, elements.len() / 2);
    assert!(!stash.is_empty());
    assert_eq!(stash.len_entries() as usize, elements.len());
    stash
}

#[test]
fn iter_over_vacant_works() {
    let stash = create_vacant_stash();
    // Test iterator over shared references.
    let mut iter = stash.iter();
    assert_eq!(iter.count(), 0);
    assert_eq!(iter.next(), None);
    // Test iterator over exclusive references.
    let mut stash = stash;
    let mut iter = stash.iter_mut();
    assert_eq!(iter.next(), None);
    // Test reverse iterator over shared references.
    let mut iter = stash.iter().rev();
    assert_eq!(iter.clone().count(), 0);
    assert_eq!(iter.next(), None);
    // Test reverse iterator over exclusive references.
    let mut stash = stash;
    let mut iter = stash.iter_mut().rev();
    assert_eq!(iter.next(), None);
}

#[test]
fn iter_over_holey_works() {
    let stash = create_holey_stash();
    // Test iterator over shared references.
    let mut iter = stash.iter();
    assert_eq!(iter.count(), 3);
    assert_eq!(iter.next(), Some(&b'B'));
    assert_eq!(iter.count(), 2);
    assert_eq!(iter.next(), Some(&b'D'));
    assert_eq!(iter.count(), 1);
    assert_eq!(iter.next(), Some(&b'F'));
    assert_eq!(iter.count(), 0);
    assert_eq!(iter.next(), None);
    // Test iterator over exclusive references.
    let mut stash = stash;
    let mut iter = stash.iter_mut();
    assert_eq!(iter.next(), Some(&mut b'B'));
    assert_eq!(iter.next(), Some(&mut b'D'));
    assert_eq!(iter.next(), Some(&mut b'F'));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.count(), 0);
}

#[test]
fn iter_rev_over_holey_works() {
    let stash = create_holey_stash();
    // Test iterator over shared references.
    let mut iter = stash.iter().rev();
    assert_eq!(iter.clone().count(), 3);
    assert_eq!(iter.next(), Some(&b'F'));
    assert_eq!(iter.clone().count(), 2);
    assert_eq!(iter.next(), Some(&b'D'));
    assert_eq!(iter.clone().count(), 1);
    assert_eq!(iter.next(), Some(&b'B'));
    assert_eq!(iter.clone().count(), 0);
    assert_eq!(iter.next(), None);
    // Test iterator over exclusive references.
    let mut stash = stash;
    let mut iter = stash.iter_mut().rev();
    assert_eq!(iter.next(), Some(&mut b'F'));
    assert_eq!(iter.next(), Some(&mut b'D'));
    assert_eq!(iter.next(), Some(&mut b'B'));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.count(), 0);
}

#[test]
fn iter_rev_works() {
    let stash = [b'A', b'B', b'C']
        .iter()
        .copied()
        .collect::<StorageStash<_>>();
    // Test iterator over shared references.
    let mut iter = stash.iter().rev();
    assert_eq!(iter.next(), Some(&b'C'));
    assert_eq!(iter.next(), Some(&b'B'));
    assert_eq!(iter.next(), Some(&b'A'));
    assert_eq!(iter.next(), None);
    // Test iterator over exclusive references.
    let mut stash = stash;
    let mut iter = stash.iter_mut().rev();
    assert_eq!(iter.next(), Some(&mut b'C'));
    assert_eq!(iter.next(), Some(&mut b'B'));
    assert_eq!(iter.next(), Some(&mut b'A'));
    assert_eq!(iter.next(), None);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct EntryMove {
    from: u32,
    to: u32,
    value: u8,
}

#[test]
fn simple_defrag_works() {
    let mut stash = [b'A', b'B', b'C', b'D', b'E', b'F']
        .iter()
        .copied()
        .collect::<StorageStash<_>>();
    assert_eq!(stash.len(), 6);
    assert_eq!(stash.len_entries(), 6);
    assert_eq!(stash.take(3), Some(b'D'));
    assert_eq!(stash.take(1), Some(b'B'));
    assert_eq!(stash.take(5), Some(b'F'));
    assert_eq!(stash.take(4), Some(b'E'));
    assert_eq!(stash.len(), 2);
    assert_eq!(stash.len_entries(), 6);
    // Now stash looks like this:
    //
    //    i | 0 | 1 | 2 | 3 | 4 | 5 |
    // next |   |   |   |   |   |   |
    // prev |   |   |   |   |   |   |
    //  val | A |   | C |   |   |   |
    //
    // After defrag the stash should look like this:
    //
    //    i | 0 | 1 |
    // next |   |   |
    // prev |   |   |
    //  val | A | C |
    let mut entry_moves = Vec::new();
    let callback = |from, to, value: &u8| {
        entry_moves.push(EntryMove {
            from,
            to,
            value: *value,
        });
    };
    assert_eq!(stash.defrag(None, callback), 4);
    assert_eq!(stash.len(), 2);
    assert_eq!(stash.len_entries(), 2);
    assert_eq!(stash.get(0), Some(&b'A'));
    assert_eq!(stash.get(1), Some(&b'C'));
    assert_eq!(
        &entry_moves,
        &[EntryMove {
            from: 2,
            to: 1,
            value: 67
        }]
    );
}

/// Returns a storage stash that looks internally like this:
///
///   i        | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7
///  ----------|---|---|---|---|---|---|---|---
///   next     |   |   |   |   |   |   |   |
///   previous |   |   |   |   |   |   |   |
///   val      |   |   |   |   | E |   |   | H
fn complex_defrag_setup() -> StorageStash<u8> {
    let mut stash = [b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H']
        .iter()
        .copied()
        .collect::<StorageStash<_>>();
    assert_eq!(stash.len(), 8);
    assert_eq!(stash.len_entries(), 8);
    // Remove some of the entries in specific order.
    assert_eq!(stash.take(0), Some(b'A'));
    assert_eq!(stash.take(6), Some(b'G'));
    assert_eq!(stash.take(1), Some(b'B'));
    assert_eq!(stash.take(5), Some(b'F'));
    assert_eq!(stash.take(2), Some(b'C'));
    assert_eq!(stash.take(3), Some(b'D'));
    assert_eq!(stash.len(), 2);
    assert_eq!(stash.len_entries(), 8);
    stash
}

/// Returns the expected entry move set for the complex defragmentation test.
fn complex_defrag_expected_moves() -> &'static [EntryMove] {
    &[
        EntryMove {
            from: 7,
            to: 0,
            value: 72,
        },
        EntryMove {
            from: 4,
            to: 1,
            value: 69,
        },
    ]
}

#[test]
fn complex_defrag_works() {
    let mut stash = complex_defrag_setup();
    let mut entry_moves = Vec::new();
    let callback = |from, to, value: &u8| {
        entry_moves.push(EntryMove {
            from,
            to,
            value: *value,
        });
    };
    assert_eq!(stash.defrag(None, callback), 6);
    // After defrag the stash should look like this:
    //
    //    i | 0 | 1 |
    // next |   |   |
    // prev |   |   |
    //  val | H | E |
    assert_eq!(stash.len(), 2);
    assert_eq!(stash.len_entries(), 2);
    assert_eq!(stash.get(0), Some(&b'H'));
    assert_eq!(stash.get(1), Some(&b'E'));
    assert_eq!(entry_moves.as_slice(), complex_defrag_expected_moves());
}

#[test]
fn incremental_defrag_works() {
    // This tests asserts that incremental defragmentation of storage stashes
    // yields the same result as immediate defragmentation of the same stash.
    let mut stash = complex_defrag_setup();
    let mut entry_moves = Vec::new();
    let mut callback = |from, to, value: &u8| {
        entry_moves.push(EntryMove {
            from,
            to,
            value: *value,
        });
    };
    let len_entries_before = stash.len_entries();
    for i in 0..stash.len_entries() {
        stash.defrag(Some(1), &mut callback);
        assert_eq!(
            stash.len_entries(),
            core::cmp::max(2, len_entries_before - i - 1)
        );
    }
    // After defrag the stash should look like this:
    //
    //    i | 0 | 1 |
    // next |   |   |
    // prev |   |   |
    //  val | H | E |
    assert_eq!(stash.len(), 2);
    assert_eq!(stash.len_entries(), 2);
    assert_eq!(stash.get(0), Some(&b'H'));
    assert_eq!(stash.get(1), Some(&b'E'));
    assert_eq!(entry_moves.as_slice(), complex_defrag_expected_moves());
}

#[derive(Debug, PartialEq, Eq)]
enum Entry {
    /// Vacant entry with `prev` and `next` links.
    Vacant(u32, u32),
    /// Occupied entry with value.
    Occupied(u8),
}

fn entries_of_stash(stash: &StorageStash<u8>) -> Vec<Entry> {
    stash
        .entries()
        .map(|entry| {
            use super::Entry as StashEntry;
            match entry {
                StashEntry::Vacant(entry) => Entry::Vacant(entry.prev, entry.next),
                StashEntry::Occupied(value) => Entry::Occupied(*value),
            }
        })
        .collect::<Vec<_>>()
}

#[test]
fn take_in_order_works() {
    let mut stash = [b'A', b'B', b'C', b'D']
        .iter()
        .copied()
        .collect::<StorageStash<_>>();
    assert_eq!(stash.len(), 4);
    assert_eq!(stash.len_entries(), 4);
    assert_eq!(stash.last_vacant_index(), None);
    assert_eq!(
        entries_of_stash(&stash),
        vec![
            Entry::Occupied(b'A'),
            Entry::Occupied(b'B'),
            Entry::Occupied(b'C'),
            Entry::Occupied(b'D')
        ]
    );
    // Take first.
    assert_eq!(stash.take(0), Some(b'A'));
    assert_eq!(stash.len(), 3);
    assert_eq!(stash.len_entries(), 4);
    assert_eq!(stash.last_vacant_index(), Some(0));
    assert_eq!(
        entries_of_stash(&stash),
        vec![
            Entry::Vacant(0, 0),
            Entry::Occupied(b'B'),
            Entry::Occupied(b'C'),
            Entry::Occupied(b'D')
        ]
    );
    // Take second.
    assert_eq!(stash.take(1), Some(b'B'));
    assert_eq!(stash.len(), 2);
    assert_eq!(stash.len_entries(), 4);
    assert_eq!(stash.last_vacant_index(), Some(0));
    assert_eq!(
        entries_of_stash(&stash),
        vec![
            Entry::Vacant(1, 1),
            Entry::Vacant(0, 0),
            Entry::Occupied(b'C'),
            Entry::Occupied(b'D')
        ]
    );
    // Take third.
    assert_eq!(stash.take(2), Some(b'C'));
    assert_eq!(stash.len(), 1);
    assert_eq!(stash.len_entries(), 4);
    assert_eq!(stash.last_vacant_index(), Some(0));
    assert_eq!(
        entries_of_stash(&stash),
        vec![
            Entry::Vacant(2, 1),
            Entry::Vacant(0, 2),
            Entry::Vacant(1, 0),
            Entry::Occupied(b'D')
        ]
    );
    // Take last.
    assert_eq!(stash.take(3), Some(b'D'));
    assert_eq!(stash.len(), 0);
    assert_eq!(stash.len_entries(), 4);
    assert_eq!(stash.last_vacant_index(), Some(0));
    assert_eq!(
        entries_of_stash(&stash),
        vec![
            Entry::Vacant(3, 1),
            Entry::Vacant(0, 2),
            Entry::Vacant(1, 3),
            Entry::Vacant(2, 0),
        ]
    );
}

#[test]
fn take_rev_order_works() {
    let mut stash = [b'A', b'B', b'C', b'D']
        .iter()
        .copied()
        .collect::<StorageStash<_>>();
    assert_eq!(stash.len(), 4);
    assert_eq!(stash.len_entries(), 4);
    assert_eq!(stash.last_vacant_index(), None);
    assert_eq!(
        entries_of_stash(&stash),
        vec![
            Entry::Occupied(b'A'),
            Entry::Occupied(b'B'),
            Entry::Occupied(b'C'),
            Entry::Occupied(b'D')
        ]
    );
    // Take last.
    assert_eq!(stash.take(3), Some(b'D'));
    assert_eq!(stash.len(), 3);
    assert_eq!(stash.len_entries(), 4);
    assert_eq!(stash.last_vacant_index(), Some(3));
    assert_eq!(
        entries_of_stash(&stash),
        vec![
            Entry::Occupied(b'A'),
            Entry::Occupied(b'B'),
            Entry::Occupied(b'C'),
            Entry::Vacant(3, 3)
        ]
    );
    // Take third.
    assert_eq!(stash.take(2), Some(b'C'));
    assert_eq!(stash.len(), 2);
    assert_eq!(stash.len_entries(), 4);
    assert_eq!(stash.last_vacant_index(), Some(2));
    assert_eq!(
        entries_of_stash(&stash),
        vec![
            Entry::Occupied(b'A'),
            Entry::Occupied(b'B'),
            Entry::Vacant(3, 3),
            Entry::Vacant(2, 2)
        ]
    );
    // Take second.
    assert_eq!(stash.take(1), Some(b'B'));
    assert_eq!(stash.len(), 1);
    assert_eq!(stash.len_entries(), 4);
    assert_eq!(stash.last_vacant_index(), Some(1));
    assert_eq!(
        entries_of_stash(&stash),
        vec![
            Entry::Occupied(b'A'),
            Entry::Vacant(3, 2),
            Entry::Vacant(1, 3),
            Entry::Vacant(2, 1)
        ]
    );
    // Take first.
    assert_eq!(stash.take(0), Some(b'A'));
    assert_eq!(stash.len(), 0);
    assert_eq!(stash.len_entries(), 4);
    assert_eq!(stash.last_vacant_index(), Some(0));
    assert_eq!(
        entries_of_stash(&stash),
        vec![
            Entry::Vacant(3, 1),
            Entry::Vacant(0, 2),
            Entry::Vacant(1, 3),
            Entry::Vacant(2, 0)
        ]
    );
}

#[test]
fn spread_layout_push_pull_works() -> ink_env::Result<()> {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let stash1 = create_holey_stash();
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&stash1, &mut KeyPtr::from(root_key));
        // Load the pushed storage vector into another instance and check that
        // both instances are equal:
        let stash2 =
            <StorageStash<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(stash1, stash2);
        Ok(())
    })
}

#[test]
#[should_panic(expected = "storage entry was empty")]
fn spread_layout_clear_works() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let stash1 = create_holey_stash();
        let root_key = Key::from([0x42; 32]);
        SpreadLayout::push_spread(&stash1, &mut KeyPtr::from(root_key));
        // It has already been asserted that a valid instance can be pulled
        // from contract storage after a push to the same storage region.
        //
        // Now clear the associated storage from `stash1` and check whether
        // loading another instance from this storage will panic since the
        // vector's length property cannot read a value:
        SpreadLayout::clear_spread(&stash1, &mut KeyPtr::from(root_key));
        let _ =
            <StorageStash<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        Ok(())
    })
    .unwrap()
}

#[test]
#[cfg(not(feature = "ink-experimental-engine"))]
fn storage_is_cleared_completely_after_pull_lazy() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        // given
        let root_key = Key::from([0x42; 32]);
        let lazy_stash = Lazy::new(create_holey_stash());
        SpreadLayout::push_spread(&lazy_stash, &mut KeyPtr::from(root_key));
        let pulled_stash = <Lazy<StorageStash<u8>> as SpreadLayout>::pull_spread(
            &mut KeyPtr::from(root_key),
        );

        // when
        SpreadLayout::clear_spread(&pulled_stash, &mut KeyPtr::from(root_key));

        // then
        let contract_id = ink_env::test::get_current_contract_account_id::<
            ink_env::DefaultEnvironment,
        >()
        .expect("Cannot get contract id");
        let storage_used = ink_env::test::count_used_storage_cells::<
            ink_env::DefaultEnvironment,
        >(&contract_id)
        .expect("used cells must be returned");
        assert_eq!(storage_used, 0);

        Ok(())
    })
    .unwrap()
}

#[test]
#[should_panic(expected = "storage entry was empty")]
#[cfg(not(feature = "ink-experimental-engine"))]
fn drop_works() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let root_key = Key::from([0x42; 32]);

        // if the setup panics it should not cause the test to pass
        let setup_result = std::panic::catch_unwind(|| {
            let stash = create_holey_stash();
            SpreadLayout::push_spread(&stash, &mut KeyPtr::from(root_key));
            let _ = <StorageStash<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(
                root_key,
            ));
            // stash is dropped which should clear the cells
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

        let _ =
            <StorageStash<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        Ok(())
    })
    .unwrap()
}

#[test]
#[cfg(feature = "ink-experimental-engine")]
fn storage_is_cleared_completely_after_pull_lazy() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        // given
        let root_key = Key::from([0x42; 32]);
        let lazy_stash = Lazy::new(create_holey_stash());
        SpreadLayout::push_spread(&lazy_stash, &mut KeyPtr::from(root_key));
        let pulled_stash = <Lazy<StorageStash<u8>> as SpreadLayout>::pull_spread(
            &mut KeyPtr::from(root_key),
        );

        // when
        SpreadLayout::clear_spread(&pulled_stash, &mut KeyPtr::from(root_key));

        // then
        let contract_id = ink_env::test::callee::<ink_env::DefaultEnvironment>();
        let storage_used = ink_env::test::count_used_storage_cells::<
            ink_env::DefaultEnvironment,
        >(&contract_id)
        .expect("used cells must be returned");
        assert_eq!(storage_used, 0);

        Ok(())
    })
    .unwrap()
}

#[test]
#[should_panic(expected = "storage entry was empty")]
#[cfg(feature = "ink-experimental-engine")]
fn drop_works() {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let root_key = Key::from([0x42; 32]);

        // if the setup panics it should not cause the test to pass
        let setup_result = std::panic::catch_unwind(|| {
            let stash = create_holey_stash();
            SpreadLayout::push_spread(&stash, &mut KeyPtr::from(root_key));
            let _ = <StorageStash<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(
                root_key,
            ));
            // stash is dropped which should clear the cells
        });
        assert!(setup_result.is_ok(), "setup should not panic");

        let contract_id = ink_env::test::callee::<ink_env::DefaultEnvironment>();
        let used_cells = ink_env::test::count_used_storage_cells::<
            ink_env::DefaultEnvironment,
        >(&contract_id)
        .expect("used cells must be returned");
        assert_eq!(used_cells, 0);

        let _ =
            <StorageStash<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        Ok(())
    })
    .unwrap()
}
