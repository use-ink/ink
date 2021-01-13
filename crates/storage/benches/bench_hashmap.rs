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

use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    BatchSize,
    Criterion,
};

use ink_primitives::Key;
use ink_storage::traits::{
    KeyPtr,
    SpreadLayout,
};

#[cfg(test)]
macro_rules! gen_tests_for_backend {
    ( $backend:ty ) => {
        criterion_group!(
            populated_cache,
            bench_insert_populated_cache,
            bench_remove_populated_cache,
        );
        criterion_group!(
            empty_cache,
            bench_insert_empty_cache,
            bench_remove_empty_cache,
        );
        criterion_main!(populated_cache, empty_cache,);

        /// The number of `ENTIRES` denotes how many test values are put into
        /// the hashmap used in these benchmarks.
        const ENTRIES: i32 = 500;

        /// Returns some test values for use in benchmarks.
        fn test_values() -> Vec<(i32, i32)> {
            (0..ENTRIES)
                .into_iter()
                .map(|index| (index, index))
                .collect::<Vec<_>>()
        }

        /// Creates a hashmap from the given slice.
        fn hashmap_from_slice(slice: &[(i32, i32)]) -> $backend {
            slice.iter().copied().collect::<$backend>()
        }

        /// Creates a hashmap from `test_values()`.
        fn setup_hashmap() -> $backend {
            let test_values = test_values();
            hashmap_from_slice(&test_values[..])
        }

        /// Returns always the same `KeyPtr`.
        fn key_ptr() -> KeyPtr {
            let root_key = Key::from([0x42; 32]);
            KeyPtr::from(root_key)
        }

        /// Creates a hashmap and pushes it to the contract storage.
        fn push_storage_hashmap() {
            let hmap = setup_hashmap();
            SpreadLayout::push_spread(&hmap, &mut key_ptr());
        }

        /// Pulls a lazily loading hashmap instance from the contract storage.
        fn pull_storage_hashmap() -> $backend {
            <$backend as SpreadLayout>::pull_spread(&mut key_ptr())
        }

        /// Iteratively checks if an entry is in the hashmap. If not, it
        /// is inserted. In either case it is incremented afterwards.
        fn insert_and_inc(hmap: &mut $backend) {
            for key in 0..ENTRIES * 2 {
                if !black_box(contains_key(hmap, &key)) {
                    black_box(insert(hmap, key, key));
                }
                *black_box(hmap.get_mut(&key)).unwrap() += 1;
            }
        }

        /// Iteratively checks if an entry is in the hashmap. If not, it
        /// is inserted. In either case it is incremented afterwards.
        ///
        /// Uses the Entry API.
        fn insert_and_inc_entry_api(hmap: &mut $backend) {
            for key in 0..ENTRIES * 2 {
                let v = black_box(hmap.entry(key).or_insert(key));
                *v += 1;
            }
        }

        /// Iteratively checks if an entry is in the hashmap. If yes, it
        /// is taken out.
        fn remove(hmap: &mut $backend) {
            for key in 0..ENTRIES * 2 {
                if black_box(contains_key(hmap, &key)) {
                    let _ = black_box(take(hmap, &key));
                }
            }
        }

        /// Iteratively checks if an entry is in the hashmap. If yes, it
        /// is taken out.
        ///
        /// Uses the Entry API.
        fn remove_entry_api(hmap: &mut $backend) {
            for key in 0..ENTRIES * 2 {
                if let Entry::Occupied(o) = black_box(hmap.entry(key)) {
                    o.remove();
                }
            }
        }

        fn bench_insert_populated_cache(c: &mut Criterion) {
            let mut group = c.benchmark_group(
                format!("{} Compare: `insert_and_inc` and `insert_and_inc_entry_api` (populated cache)", stringify!($backend))
            );
            group.bench_function("insert_and_inc", |b| {
                b.iter_batched_ref(
                    || setup_hashmap(),
                    |hmap| insert_and_inc(hmap),
                    BatchSize::SmallInput,
                )
            });
            group.bench_function("insert_and_inc_entry_api", |b| {
                b.iter_batched_ref(
                    || setup_hashmap(),
                    |hmap| insert_and_inc_entry_api(hmap),
                    BatchSize::SmallInput,
                )
            });
            group.finish();
        }

        fn bench_remove_populated_cache(c: &mut Criterion) {
            let _ = ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
                let mut group = c.benchmark_group(
                    format!("{} Compare: `remove` and `remove_entry_api` (populated cache)", stringify!($backend))
                );
                group.bench_function("remove", |b| {
                    b.iter_batched_ref(
                        || setup_hashmap(),
                        |hmap| remove(hmap),
                        BatchSize::SmallInput,
                    )
                });
                group.bench_function("remove_entry_api", |b| {
                    b.iter_batched_ref(
                        || setup_hashmap(),
                        |hmap| remove_entry_api(hmap),
                        BatchSize::SmallInput,
                    )
                });
                group.finish();
                Ok(())
            })
            .unwrap();
        }

        fn bench_insert_empty_cache(c: &mut Criterion) {
            let _ = ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
                let mut group = c.benchmark_group(
                    format!("{} Compare: `insert_and_inc` and `insert_and_inc_entry_api` (empty cache)", stringify!($backend))
                );
                group.bench_function("insert_and_inc", |b| {
                    b.iter_batched_ref(
                        || {
                            push_storage_hashmap();
                            pull_storage_hashmap()
                        },
                        |hmap| insert_and_inc(hmap),
                        BatchSize::SmallInput,
                    )
                });
                group.bench_function("insert_and_inc_entry_api", |b| {
                    b.iter_batched_ref(
                        || {
                            push_storage_hashmap();
                            pull_storage_hashmap()
                        },
                        |hmap| insert_and_inc_entry_api(hmap),
                        BatchSize::SmallInput,
                    )
                });
                group.finish();
                Ok(())
            })
            .unwrap();
        }

        fn bench_remove_empty_cache(c: &mut Criterion) {
            let _ = ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
                let mut group =
                    c.benchmark_group(format!("{} Compare: `remove` and `remove_entry_api` (empty cache)", stringify!($backend)));
                group.bench_function("remove", |b| {
                    b.iter_batched_ref(
                        || {
                            push_storage_hashmap();
                            pull_storage_hashmap()
                        },
                        |hmap| remove(hmap),
                        BatchSize::SmallInput,
                    )
                });
                group.bench_function("remove_entry_api", |b| {
                    b.iter_batched_ref(
                        || {
                            push_storage_hashmap();
                            pull_storage_hashmap()
                        },
                        |hmap| remove_entry_api(hmap),
                        BatchSize::SmallInput,
                    )
                });
                group.finish();
                Ok(())
            })
            .unwrap();
        }
    };
}

mod lazyhmap_backend {
    use super::*;
    use ink_env::hash::Blake2x256;
    use ink_storage::lazy::lazy_hmap::{
        Entry,
        LazyHashMap,
    };

    gen_tests_for_backend!(LazyHashMap<i32, i32, Blake2x256>);

    pub fn insert(
        hmap: &mut LazyHashMap<i32, i32, Blake2x256>,
        key: i32,
        value: i32,
    ) -> Option<i32> {
        hmap.put_get(&key, Some(value))
    }

    pub fn take(hmap: &mut LazyHashMap<i32, i32, Blake2x256>, key: &i32) -> Option<i32> {
        hmap.put_get(key, None)
    }

    pub fn contains_key(hmap: &LazyHashMap<i32, i32, Blake2x256>, key: &i32) -> bool {
        hmap.get(key).is_some()
    }

    pub fn run() {
        self::main()
    }
}

mod hashmap_backend {
    use super::*;
    use ink_storage::collections::{
        hashmap::Entry,
        HashMap as StorageHashMap,
    };

    gen_tests_for_backend!(StorageHashMap<i32, i32>);

    pub fn insert(
        hmap: &mut StorageHashMap<i32, i32>,
        key: i32,
        value: i32,
    ) -> Option<i32> {
        hmap.insert(key, value)
    }

    pub fn take(hmap: &mut StorageHashMap<i32, i32>, key: &i32) -> Option<i32> {
        hmap.take(key)
    }

    pub fn contains_key(hmap: &StorageHashMap<i32, i32>, key: &i32) -> bool {
        hmap.contains_key(key)
    }

    pub fn run() {
        self::main()
    }
}

fn main() {
    hashmap_backend::run();
    lazyhmap_backend::run();
}
