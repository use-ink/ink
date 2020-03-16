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

use crate::{
    env::{
        self,
        Result,
    },
    storage::{
        alloc::{
            AllocateUsing,
            BumpAlloc,
            Initialize,
        },
        btree_map::impls::{
            Entry,
            CAPACITY,
        },
        collections::btree_map::{
            node::NodeHandle,
            test_utils::*,
        },
        BTreeMap,
    },
};
use ink_prelude::collections::HashMap;
use ink_primitives::Key;
use itertools::Itertools;

#[test]
fn empty_map_works() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let map = empty_map();

        // then
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        Ok(())
    })
}

#[test]
fn remove_element_from_empty_map() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut map = empty_map();
        assert_eq!(map.remove(&4), None);
        Ok(())
    })
}

#[test]
fn insert_into_empty_map_works() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut map = empty_map();

        // when
        assert_eq!(map.insert(0, 10), None);

        // then
        assert_eq!(map.get(&0), Some(&10));
        assert_eq!(map.contains_key(&0), true);
        assert_eq!(map.get_key_value(&0), Some((&0, &10)));
        Ok(())
    })
}

#[test]
fn remove_works() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut map = empty_map();

        // when
        assert_eq!(map.insert(4, 40), None);
        assert_eq!(map.get(&4), Some(&40));
        assert_eq!(map.len(), 1);

        // then
        assert_eq!(map.remove(&4), Some(40));
        assert_eq!(map.get(&4), None);
        assert_eq!(map.len(), 0);
        Ok(())
    })
}

#[test]
fn multiple_inserts_for_same_key_work() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut map = empty_map();
        assert_eq!(map.insert(0, 10), None);

        // when
        assert_eq!(map.insert(0, 20), Some(10));
        assert_eq!(map.get(&0), Some(&20));
        assert_eq!(map.len(), 1);

        // then
        assert_eq!(map.remove(&0), Some(20));
        assert_eq!(map.get(&0), None);
        assert_eq!(map.len(), 0);
        Ok(())
    })
}

#[test]
fn putting_on_same_key_works() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut map = empty_map();

        // when
        assert_eq!(map.insert(42, 420), None);
        assert_eq!(map.len(), 1);
        assert_eq!(map.insert(42, 520), Some(420));

        // then
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&42), Some(&520));
        Ok(())
    })
}

#[test]
fn filled_map_must_work() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut map = filled_map();
        assert_eq!(map.get(&5), Some(&50));
        assert_eq!(map.get(&42), Some(&420));
        assert_eq!(map.get(&1337), Some(&13370));
        assert_eq!(map.get(&77), Some(&770));
        assert_eq!(map.get(&4), None);
        assert_eq!(map.len(), 4);

        // when
        assert_eq!(map.insert(4, 40), None);

        // then
        assert_eq!(map.get(&4), Some(&40));
        assert_eq!(map.len(), 5);
        Ok(())
    })
}

#[test]
fn tree_must_remain_balanced() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut map = empty_map();
        for i in 0..CAPACITY as i32 {
            map.insert(i, i * 10);
        }
        assert_eq!(map.len(), CAPACITY as u32);
        assert_eq!(map.node_count(), 1);

        // when
        // when one element more is inserted
        let one_more = CAPACITY as i32;
        assert_eq!(map.insert(one_more, one_more * 10), None);

        // then
        // the node count has to jump to three, since we then must
        // have two children and a parent node.
        assert_eq!(map.len(), CAPACITY as u32 + 1);
        assert_eq!(map.node_count(), 3);

        // when
        // when the element is removed again the node count has
        // to go back to one since the nodes should have been merged.
        assert_eq!(map.remove(&one_more), Some(one_more * 10));
        assert_eq!(map.len(), 11);

        // then
        assert_eq!(map.node_count(), 1);
        assert!(map.get_node(NodeHandle::new(0)).is_some());
        Ok(())
    })
}

#[test]
fn entry_api_works_with_empty_map() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut map = empty_map();
        let key = 5;
        let val = 50;

        // when
        let entry = map.entry(key);
        match entry {
            Entry::Vacant(v) => {
                v.insert(val);
            }
            Entry::Occupied(_) => {
                unreachable!("map is created as empty");
            }
        }

        // then
        assert_eq!(map.get(&key), Some(&val));
        assert_eq!(map.contains_key(&key), true);
        assert_eq!(map.get_key_value(&key), Some((&key, &val)));
        Ok(())
    })
}

#[test]
fn entry_api_works_with_filled_map() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut map = filled_map();
        assert_eq!(map.entry(5).key(), &5);
        assert_eq!(map.entry(-1).key(), &-1);
        assert_eq!(map.entry(997).or_insert(9970), &9970);
        Ok(())
    })
}

#[test]
fn entry_api_works_with_strings_and_multiple_calls() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut map = unsafe {
            let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
            BTreeMap::allocate_using(&mut alloc).initialize_into(())
        };
        let k = String::from("poneyland");
        map.entry(k.clone()).or_insert(12);

        // when
        match map.entry(k.clone()) {
            Entry::Occupied(mut o) => {
                *o.get_mut() += 10;
                assert_eq!(*o.get(), 22);

                *o.get_mut() += 2;
            }
            _ => unreachable!(),
        };

        // then
        assert_eq!(map.get(&k).expect("must be there"), &24);
        Ok(())
    })
}

#[test]
fn putting_and_removing_many_items_works() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let count = CAPACITY as i32 * 100;
        let mut map = empty_map();
        let mut len = map.len();
        for i in 0..count {
            assert_eq!(map.insert(i, i * 10), None);
            len += 1;
            assert_eq!(map.len(), len);
        }
        let max_node_count = map.node_count();

        // when
        for i in 0..count {
            assert_eq!(map.get(&i), Some(&(i * 10)));
            assert_eq!(map.remove(&i), Some(i * 10));
            assert_eq!(map.get(&i), None);
            len -= 1;
            assert_eq!(map.len(), len);
        }

        // then
        assert_eq!(map.len(), 0);
        assert_eq!(map.node_count(), 0);
        assert!(storage_empty(&map, max_node_count));
        Ok(())
    })
}

#[test]
fn simple_insert_and_removal() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut map = empty_map();
        let xs = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let mut len = 0;
        xs.iter().for_each(|i| {
            if let Some(_) = map.insert(*i, i * 10) {
                unreachable!("no element must already exist there");
            }
            assert_eq!(map.get(&i), Some(&(i * 10)));
            len += 1;
            assert_eq!(map.len(), len);
        });
        xs.iter().for_each(|k| {
            let v = *k * 10;
            assert_eq!(map.get(k), Some(&v));
            assert_eq!(map.contains_key(k), true);
            assert_eq!(map.get_key_value(k), Some((k, &v)));
        });
        let max_node_count = map.node_count();

        // when
        // remove all
        xs.iter().for_each(|i| {
            match map.remove(&i) {
                Some(v) => {
                    assert_eq!(v, i * 10);
                    len -= 1;
                }
                None => unreachable!(),
            };
            assert_eq!(map.len(), len);
        });

        // then
        assert_eq!(map.len(), 0);
        assert_eq!(map.node_count(), 0);
        assert!(storage_empty(&map, max_node_count));
        Ok(())
    })
}

#[test]
fn alternating_inserts_and_remove_works() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut map = empty_map();
        let mut len = map.len();
        let ops = vec![
            CAPACITY as i32 * 20, // insert
            CAPACITY as i32 * 10, // remove
            CAPACITY as i32 * 10, // insert
            CAPACITY as i32 * 20, // remove
        ];
        let mut max_node_count = 0;

        // when
        ops.iter().enumerate().for_each(|(p, n)| {
            if p % 2 == 0 {
                // if it's an even array index we insert `n` elements
                for i in 1..*n {
                    assert_eq!(map.insert(i, i * 10), None);
                    assert_eq!(map.get(&i), Some(&(i * 10)));
                    len += 1;
                    assert_eq!(map.len(), len);

                    let nodes = map.node_count();
                    if nodes > max_node_count {
                        max_node_count = nodes;
                    }
                }
            } else {
                // on odd indices we remove `n` elements
                for i in 1..*n {
                    assert_eq!(map.remove(&i), Some(i * 10));
                    assert_eq!(map.get(&i), None);
                    len -= 1;
                    assert_eq!(map.len(), len);
                }
            }
        });

        // then
        assert_eq!(map.len(), 0);
        assert_eq!(map.node_count(), 0);
        assert!(storage_empty(&map, max_node_count));
        Ok(())
    })
}

#[test]
fn sorted_insert_and_removal() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let mut map = empty_map();
        let mut len = map.len();
        let xs = vec![
            -95, -89, -86, -67, -54, -13, -6, -1, 4, 13, 15, 21, 31, 40, 65,
        ];
        let mut xs = xs.clone();
        xs.sort_by(|a, b| a.cmp(b));
        xs = xs.into_iter().unique().collect();
        let mut max_node_count = 0;

        // first insert in sorted order
        xs.iter().for_each(|i| {
            assert_eq!(map.insert(*i, i * 10), None);
            len += 1;
            max_node_count += map.node_count();
            assert_eq!(map.len(), len);
            assert!(every_edge_exists_only_once(&map));
        });

        // when
        // remove from the back
        xs.sort_by(|a, b| b.cmp(a));
        xs.iter().for_each(|i| {
            assert_eq!(map.remove(&i), Some(i * 10));
            len -= 1;
            assert_eq!(map.len(), len);
            assert!(every_edge_exists_only_once(&map));
        });

        // then
        assert_eq!(map.len(), 0);
        assert_eq!(map.node_count(), 0);
        assert!(storage_empty(&map, max_node_count));
        Ok(())
    })
}

/// These are some cases which in the past have shown to generate complex trees
/// for which the removal/insert operations touch all kinds of corner cases.
#[test]
fn complex_trees_work() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let xs = [
            -72, -68, 36, -30, 0, -38, -74, -60, 4, -2, 28, -34, 60, -42, -14, 32, -48,
            18, -6, 24, -10, 40, 62, -64, 48, -56, 14, 3,
        ];
        insert_and_remove(xs.to_vec());

        let xs = [
            2, -30, -26, 0, -34, -4, -38, -42, -8, -56, 66, 34, 16, 36, -62, -12, -20,
            38, 30, -50, -66, 6, 70, 62, -16, 12, -70, 42, 31,
        ];
        insert_and_remove(xs.to_vec());

        let xs = [-2, -66, -44, 34, -6, 62, 2, 6, -30, -70, 30, -62, 7, -44, 7];
        insert_and_remove(xs.to_vec());
        Ok(())
    })
}

#[test]
fn extend_works() -> Result<()> {
    env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // given
        let arr1 = [(1i32, 2i32), (3, 4), (5, 6)];
        let arr2 = [(7i32, 8i32), (9, 10)];
        let mut map = empty_map();

        let mut expected = HashMap::new();

        expected.extend(arr1.iter().cloned());
        expected.extend(arr2.iter().cloned());

        // when
        map.extend(arr1.iter());
        map.extend(arr2.iter());

        // then
        assert_eq!(map.len() as usize, expected.len());
        for (k, v) in &expected {
            assert_eq!(Some(v), map.get(k))
        }
        Ok(())
    })
}
