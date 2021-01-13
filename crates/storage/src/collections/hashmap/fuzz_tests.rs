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
use crate::traits::{
    KeyPtr,
    SpreadLayout,
};
use ink_primitives::Key;
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
fn insert_and_remove(xs: Vec<i32>, inserts_each: u8) -> StorageHashMap<i32, i32> {
    let mut map = <StorageHashMap<i32, i32>>::new();
    let mut cnt_inserts = 0;
    let mut previous_even_x = None;
    let inserts_each = inserts_each as i32;

    for x in 0..xs.len() as i32 {
        if x % 2 == 0 {
            // On even numbers we insert
            for key in x..x + inserts_each {
                let val = key * 10;
                if map.insert(key, val).is_none() {
                    assert_eq!(map.get(&key), Some(&val));
                    cnt_inserts += 1;
                }
                assert_eq!(map.len(), cnt_inserts);
            }
            if previous_even_x.is_none() {
                previous_even_x = Some(x);
            }
        } else if previous_even_x.is_some() {
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
    }
    map
}

#[quickcheck]
fn fuzz_inserts_and_removes(xs: Vec<i32>, inserts_each: u8) {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let _ = insert_and_remove(xs, inserts_each);
        Ok(())
    })
    .unwrap()
}

/// Inserts all elements from `xs`. Then removes each `xth` element from the map
/// and asserts that all non-`xth` elements are still in the map.
#[quickcheck]
fn fuzz_removes(xs: Vec<i32>, xth: usize) {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        // given
        let xs: Vec<i32> = xs.into_iter().unique().collect();
        let xth = xth.max(1);
        let mut map = <StorageHashMap<i32, i32>>::new();
        let mut len = map.len();

        // when
        // 1) insert all
        for x in 0..xs.len() {
            let i = xs.get(x).expect(
                "x is always in bounds since we iterate over the vec length; qed",
            );
            assert_eq!(map.insert(*i, i * 10), None);
            len += 1;
            assert_eq!(map.len(), len);
        }

        // 2) remove every `xth` element of `xs` from the map
        for x in 0..xs.len() {
            if x % xth == 0 {
                let i = xs.get(x).expect(
                    "x is always in bounds since we iterate over the vec length; qed",
                );
                assert_eq!(map.take(&i), Some(i * 10));
                len -= 1;
            }
            assert_eq!(map.len(), len);
        }

        // then
        // everything else must still be get-able
        for x in 0..xs.len() {
            if x % xth != 0 {
                let i = xs.get(x).expect(
                    "x is always in bounds since we iterate over the vec length; qed",
                );
                assert_eq!(map.get(&i), Some(&(i * 10)));
            }
        }

        Ok(())
    })
    .unwrap()
}

#[quickcheck]
fn fuzz_defrag(xs: Vec<i32>, inserts_each: u8) {
    ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        // Create a `HashMap<i32, i32>` and execute some pseudo-randomized
        // insert/remove operations on it.
        let mut map = insert_and_remove(xs, inserts_each);

        // Build a collection of the keys/values in this hash map
        let kv_pairs: Vec<(i32, i32)> = map
            .keys
            .iter()
            .map(|key| {
                (
                    key.to_owned(),
                    map.get(key).expect("value must exist").to_owned(),
                )
            })
            .collect();
        assert_eq!(map.len(), kv_pairs.len() as u32);

        // Then defragment the hash map
        map.defrag(None);

        // Then we push the defragmented hash map to storage and pull it again
        let root_key = Key::from([0x00; 32]);
        SpreadLayout::push_spread(&map, &mut KeyPtr::from(root_key));
        let map2: StorageHashMap<i32, i32> =
            SpreadLayout::pull_spread(&mut KeyPtr::from(root_key));

        // Assert that everything that should be is still in the hash map
        assert_eq!(map2.len(), kv_pairs.len() as u32);
        for (key, val) in kv_pairs {
            assert_eq!(map2.get(&key), Some(&val));
        }

        Ok(())
    })
    .unwrap()
}
