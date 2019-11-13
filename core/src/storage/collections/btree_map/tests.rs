// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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
    storage::{
        alloc::{
            AllocateUsing,
            BumpAlloc,
            Initialize,
        },
        Key,
        BTreeMap,
    },
    test_utils::run_test,
};
use crate::storage::btree_map::impls::Entry;

fn empty_map() -> BTreeMap<i32, i32> {
    unsafe {
        let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
        BTreeMap::allocate_using(&mut alloc).initialize_into(())
    }
}

fn filled_map() -> BTreeMap<i32, i32> {
    let mut map = empty_map();
    map.insert(5, 50);
    map.insert(42, 420);
    map.insert(1337, 13370);
    map.insert(77, 770);
    assert_eq!(map.len(), 4);
    map
}

#[test]
fn new_unchecked() {
    run_test(|| {
        let map = empty_map();
        // Initial invariant.
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        //assert_eq!(stash.iter().next(), None);
    })
}

#[test]
fn put_empty() {
    run_test(|| {
        let mut map = empty_map();
        // Before and after first put.
        assert_eq!(map.insert(42, 420), None);
        assert_eq!(map.insert(42, 520), Some(420));
        assert_eq!(map.get(&42), Some(&520));
    })
}

#[test]
fn first_put_filled() {
    run_test(|| {
        let mut map = filled_map();
        assert_eq!(map.get(&5), Some(&50));

        assert_eq!(map.get(&42), Some(&420));
        assert_eq!(map.get(&1337), Some(&13370));
        assert_eq!(map.get(&77), Some(&770));
        assert_eq!(map.get(&4), None);
        assert_eq!(map.len(), 4);

        assert_eq!(map.insert(4, 40), None);

        assert_eq!(map.get(&4), Some(&40));
        assert_eq!(map.len(), 5);
    })
}

#[test]
fn put_filled2() {
    run_test(|| {
        let mut map = empty_map();
        let mut len  = map.len();
        for i in 1..200 {
            assert_eq!(map.insert(i, i * 10), None);
            len += 1;
            assert_eq!(map.len(), len);
        }

        for i in 1..200 {
            assert_eq!(map.get(&i), Some(&(i * 10)));
        }
    })
}

#[test]
fn entry_api() {
    run_test(|| {
        let mut map = filled_map();
        assert_eq!(map.entry(5).key(), &5);
        assert_eq!(map.entry(-1).key(), &-1);

        assert_eq!(map.entry(997).or_insert(9970), &9970);
    });
}

#[test]
fn entry_api2() {
    run_test(|| {
        let mut map = unsafe {
            let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
            BTreeMap::allocate_using(&mut alloc).initialize_into(())
        };
        map.entry(String::from("poneyland")).or_insert(12);
        let p = String::from("poneyland");
        if let Entry::Occupied(mut o) = map.entry(p) {
            *o.get_mut() += 10;
            assert_eq!(*o.get(), 22);

            // We can use the same Entry multiple times.
            *o.get_mut() += 2;
        }
        let p = String::from("poneyland");
        assert_eq!(map.get(&p).expect("must be there"), &24);
    });
}
