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

use crate::traits::{
    KeyPtr,
    SpreadLayout,
};
use ink_primitives::Key;

#[cfg(test)]
macro_rules! gen_tests_for_backend {
    ( $backend:ty ) => {
        /// Returns some test values.
        fn test_values() -> [(u8, i32); 2] {
            [(b'A', 13), (b'B', 23)]
        }

        /// Returns a prefilled hashmap with `[('A', 13), ['B', 23])`.
        fn prefilled_hmap() -> $backend {
            test_values().iter().copied().collect::<$backend>()
        }

        /// Returns always the same `KeyPtr`.
        fn key_ptr() -> KeyPtr {
            let root_key = Key::from([0x42; 32]);
            KeyPtr::from(root_key)
        }

        /// Pushes a `HashMap` instance into the contract storage.
        fn push_hmap(hmap: &$backend) {
            SpreadLayout::push_spread(hmap, &mut key_ptr());
        }

        /// Pulls a `HashMap` instance from the contract storage.
        fn pull_hmap() -> $backend {
            <$backend as SpreadLayout>::pull_spread(&mut key_ptr())
        }

        fn push_pull_prefilled_hmap() -> $backend {
            let hmap1 = prefilled_hmap();
            assert_eq!(hmap1.get(&b'A'), Some(&13));
            push_hmap(&hmap1);
            pull_hmap()
        }

        #[test]
        fn insert_inexistent_works_with_empty() {
            // given
            let mut hmap = <$backend>::new();
            assert!(matches!(hmap.entry(b'A'), Vacant(_)));
            assert!(hmap.get(&b'A').is_none());

            // when
            assert_eq!(*hmap.entry(b'A').or_insert(77), 77);

            // then
            assert_eq!(hmap.get(&b'A'), Some(&77));
            assert_eq!(hmap.len_cached_entries(), 1);
        }

        #[test]
        fn insert_existent_works() {
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
            assert_eq!(hmap.len_cached_entries(), 2);
        }

        #[test]
        fn mutations_work_with_push_pull() -> ink_env::Result<()> {
            ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
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
        fn simple_insert_with_works() {
            // given
            let mut hmap = prefilled_hmap();

            // when
            assert!(hmap.get(&b'C').is_none());
            let v = hmap.entry(b'C').or_insert_with(|| 42);

            // then
            assert_eq!(*v, 42);
            assert_eq!(hmap.get(&b'C'), Some(&42));
            assert_eq!(hmap.len_cached_entries(), 3);
        }

        #[test]
        fn simple_default_insert_works() {
            // given
            let mut hmap = <$backend>::new();

            // when
            let v = hmap.entry(b'A').or_default();

            // then
            assert_eq!(*v, 0);
            assert_eq!(hmap.get(&b'A'), Some(&0));
        }

        #[test]
        fn insert_with_works_with_mutations() {
            // given
            let mut hmap = <$backend>::new();
            let v = hmap.entry(b'A').or_insert_with(|| 42);
            assert_eq!(*v, 42);

            // when
            *v += 1;

            // then
            assert_eq!(hmap.get(&b'A'), Some(&43));
            assert_eq!(hmap.len_cached_entries(), 1);
        }

        #[test]
        fn insert_with_works_with_push_pull() -> ink_env::Result<()> {
            ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
                // given
                let mut hmap1 = <$backend>::new();
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
        fn simple_insert_with_key_works() {
            // given
            let mut hmap = <$backend>::new();

            // when
            let _ = hmap.entry(b'A').or_insert_with_key(|key| (key * 2).into());

            // then
            assert_eq!(hmap.get(&b'A'), Some(&130));
        }

        #[test]
        fn key_get_works_with_nonexistent() {
            let mut hmap = <$backend>::new();
            assert_eq!(hmap.entry(b'A').key(), &b'A');
        }

        #[test]
        fn key_get_works_with_existent() {
            let mut hmap = prefilled_hmap();
            assert_eq!(hmap.entry(b'A').key(), &b'A');
            assert_eq!(hmap.entry(b'B').key(), &b'B');
        }

        #[test]
        fn and_modify_has_no_effect_for_nonexistent() {
            // given
            let mut hmap = <$backend>::new();

            // when
            hmap.entry(b'B').and_modify(|e| *e += 1).or_insert(42);

            // then
            assert_eq!(hmap.get(&b'B'), Some(&42));
        }

        #[test]
        fn and_modify_works_for_existent() {
            // given
            let mut hmap = prefilled_hmap();

            // when
            assert_eq!(hmap.get(&b'B'), Some(&23));
            hmap.entry(b'B').and_modify(|e| *e += 1).or_insert(7);

            // then
            assert_eq!(hmap.get(&b'B'), Some(&24));
        }

        #[test]
        fn occupied_entry_api_works_with_push_pull() -> ink_env::Result<()> {
            ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
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
        fn vacant_api_works() {
            let mut hmap = <$backend>::new();
            match hmap.entry(b'A') {
                Entry::Occupied(_) => panic!(),
                Entry::Vacant(v) => {
                    assert_eq!(v.key(), &b'A');
                    assert_eq!(v.into_key(), b'A');
                }
            }
        }

        #[test]
        fn vacant_api_works_with_push_pull() -> ink_env::Result<()> {
            ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
                // given
                let mut hmap1 = <$backend>::new();
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

        #[test]
        fn pulling_occupied_entry_must_succeed() -> ink_env::Result<()> {
            ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
                // given
                let hmap1 = prefilled_hmap();
                push_hmap(&hmap1);

                // when
                let mut hmap2 = pull_hmap();

                // then
                for (k, _v) in test_values().iter() {
                    match hmap2.entry(*k) {
                        Entry::Occupied(_) => (),
                        Entry::Vacant(_) => panic!("the entry must be occupied"),
                    }
                }
                Ok(())
            })
        }

        #[test]
        fn value_not_in_cache_but_in_storage_get_and_get_mut() -> ink_env::Result<()> {
            ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
                // given
                let mut hmap2 = push_pull_prefilled_hmap();

                // then
                match hmap2.entry(b'A') {
                    Entry::Occupied(mut o) => {
                        assert_eq!(o.get(), &13);
                        assert_eq!(o.get_mut(), &mut 13);
                    }
                    Entry::Vacant(_) => panic!(),
                }

                Ok(())
            })
        }

        #[test]
        fn value_not_in_cache_but_in_storage_insert() -> ink_env::Result<()> {
            ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
                // given
                let mut hmap2 = push_pull_prefilled_hmap();

                // then
                match hmap2.entry(b'A') {
                    Entry::Occupied(mut o) => {
                        assert_eq!(o.insert(999), 13);
                    }
                    Entry::Vacant(_) => panic!(),
                }
                assert_eq!(hmap2.get(&b'A'), Some(&999));

                Ok(())
            })
        }

        #[test]
        fn value_not_in_cache_but_in_storage_remove_entry() -> ink_env::Result<()> {
            ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
                // given
                let mut hmap2 = push_pull_prefilled_hmap();

                // then
                match hmap2.entry(b'A') {
                    Entry::Occupied(o) => {
                        assert_eq!(o.remove_entry(), (b'A', 13));
                        assert_eq!(hmap2.get(&b'A'), None);
                        push_hmap(&hmap2);
                    }
                    Entry::Vacant(_) => panic!(),
                }

                let hmap3 = pull_hmap();
                assert_eq!(hmap3.get(&b'A'), None);
                Ok(())
            })
        }

        #[test]
        fn value_not_in_cache_is_properly_flushed_after_insert() -> ink_env::Result<()> {
            ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
                // given
                let mut hmap2 = push_pull_prefilled_hmap();

                // when
                match hmap2.entry(b'A') {
                    Entry::Occupied(mut o) => {
                        assert_eq!(o.insert(999), 13);
                    }
                    Entry::Vacant(_) => panic!(),
                }
                assert_eq!(hmap2.get(&b'A'), Some(&999));
                push_hmap(&hmap2);

                // then
                // the value must have been flushed, which implies that after the
                // insert is was marked as `Mutated`.
                let hmap3 = pull_hmap();
                assert_eq!(hmap3.get(&b'A'), Some(&999));

                Ok(())
            })
        }

        #[test]
        fn value_not_in_cache_but_in_storage_into_mut() -> ink_env::Result<()> {
            ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
                // given
                let mut hmap2 = push_pull_prefilled_hmap();

                // then
                match hmap2.entry(b'A') {
                    Entry::Occupied(o) => assert_eq!(o.into_mut(), &mut 13),
                    Entry::Vacant(_) => panic!(),
                }

                Ok(())
            })
        }
    };
}

mod lazyhmap_backend {
    use super::*;
    use crate::lazy::lazy_hmap::{
        Entry,
        Entry::{
            Occupied,
            Vacant,
        },
        LazyHashMap,
    };
    use ink_env::hash::Blake2x256;

    gen_tests_for_backend!(LazyHashMap<u8, i32, Blake2x256>);
}

mod hashmap_backend {
    use super::*;
    use crate::collections::hashmap::{
        Entry,
        Entry::{
            Occupied,
            Vacant,
        },
        HashMap as StorageHashMap,
    };
    use ink_env::hash::Blake2x256;

    gen_tests_for_backend!(StorageHashMap<u8, i32, Blake2x256>);
}
