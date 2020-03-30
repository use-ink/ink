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

use super::Stash as StorageStash;

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
fn get_works() {
    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    let mut stash = test_values.iter().copied().collect::<StorageStash<_>>();
    for (index, expected_value) in test_values.iter().enumerate() {
        assert_eq!(stash.get(index as u32), Some(expected_value));
        assert_eq!(stash.get_mut(index as u32), Some(*expected_value).as_mut());
    }
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
    assert_eq!(iter.next(), Some(&b'A'));
    assert_eq!(iter.next(), Some(&b'B'));
    assert_eq!(iter.next(), Some(&b'C'));
    assert_eq!(iter.next(), None);
    // Test iterator over exclusive references.
    let mut stash = stash;
    let mut iter = stash.iter_mut();
    assert_eq!(iter.next(), Some(&mut b'A'));
    assert_eq!(iter.next(), Some(&mut b'B'));
    assert_eq!(iter.next(), Some(&mut b'C'));
    assert_eq!(iter.next(), None);
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
    //    i | 0 | 1 | 2 | 3 | 4 | 5 | 6 |
    // next |   |   |   |   |   |   |   |
    // prev |   |   |   |   |   |   |   |
    //  val | A |   | C |   |   |   |   |
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
    stash.defrag(None, callback);
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
///    i | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
/// next |   |   |   |   |   |   |   |   |
/// prev |   |   |   |   |   |   |   |   |
///  val |   |   |   |   | E |   |   | H |
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
    stash.defrag(None, callback);
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
