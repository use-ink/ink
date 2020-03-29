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

#[test]
fn defrag_works() {
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
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    struct EntryMove {
        from: u32,
        to: u32,
        value: u8,
    }
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
    assert_eq!(&entry_moves, &[EntryMove { from: 2, to: 1, value: 67 }]);
}
