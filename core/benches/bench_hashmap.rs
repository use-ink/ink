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

use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    BatchSize,
    Criterion,
};

use ink_core::{
    env,
    storage2::{
        collections::{
            hashmap::Entry,
            HashMap as StorageHashMap,
        },
        traits::{
            KeyPtr,
            SpreadLayout,
        },
    },
};
use ink_primitives::Key;

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
/// the `StorageHashMap` used in these benchmarks.
const ENTRIES: i32 = 500;

/// Returns some test values for use in benchmarks.
fn test_values() -> Vec<(i32, i32)> {
    (0..ENTRIES)
        .into_iter()
        .map(|index| (index, index))
        .collect::<Vec<_>>()
}

/// Creates a `StorageHashMap` from the given slice.
fn hashmap_from_slice(slice: &[(i32, i32)]) -> StorageHashMap<i32, i32> {
    slice.iter().copied().collect::<StorageHashMap<i32, i32>>()
}

/// Creates a `StorageHashMap` from `test_values()`.
fn setup_hashmap() -> StorageHashMap<i32, i32> {
    let test_values = test_values();
    hashmap_from_slice(&test_values[..])
}

/// Returns always the same `KeyPtr`.
fn key_ptr() -> KeyPtr {
    let root_key = Key::from([0x42; 32]);
    KeyPtr::from(root_key)
}

/// Creates a `StorageHashMap` and pushes it to the contract storage.
fn push_storage_hashmap() {
    let hmap = setup_hashmap();
    SpreadLayout::push_spread(&hmap, &mut key_ptr());
}

/// Pulls a lazily loading `StorageHashMap` instance from the contract storage.
fn pull_storage_hashmap() -> StorageHashMap<i32, i32> {
    <StorageHashMap<i32, i32> as SpreadLayout>::pull_spread(&mut key_ptr())
}

/// Iteratively checks if an entry is in the `StorageHashMap`. If not, it
/// is inserted. In either case it is incremented afterwards.
fn insert_and_inc(hmap: &mut StorageHashMap<i32, i32>) {
    for key in 0..ENTRIES * 2 {
        if !black_box(hmap.contains_key(&key)) {
            black_box(hmap.insert(key, key));
        }
        *black_box(hmap.get_mut(&key)).unwrap() += 1;
    }
}

/// Iteratively checks if an entry is in the `StorageHashMap`. If not, it
/// is inserted. In either case it is incremented afterwards.
///
/// Uses the Entry API.
fn insert_and_inc_entry_api(hmap: &mut StorageHashMap<i32, i32>) {
    for key in 0..ENTRIES * 2 {
        let v = black_box(hmap.entry(key).or_insert(key));
        *v += 1;
    }
}

/// Iteratively checks if an entry is in the `StorageHashMap`. If yes, it
/// is taken out.
fn remove(hmap: &mut StorageHashMap<i32, i32>) {
    for key in 0..ENTRIES * 2 {
        if black_box(hmap.contains_key(&key)) {
            let _ = black_box(hmap.take(&key));
        }
    }
}

/// Iteratively checks if an entry is in the `StorageHashMap`. If yes, it
/// is taken out.
///
/// Uses the Entry API.
fn remove_entry_api(hmap: &mut StorageHashMap<i32, i32>) {
    for key in 0..ENTRIES * 2 {
        if let Entry::Occupied(o) = black_box(hmap.entry(key)) {
            o.remove();
        }
    }
}

fn bench_insert_populated_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group(
        "Compare: `insert_and_inc` and `insert_and_inc_entry_api` (populated cache)",
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
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut group = c.benchmark_group(
            "Compare: `remove` and `remove_entry_api` (populated cache)",
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
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut group = c.benchmark_group(
            "Compare: `insert_and_inc` and `insert_and_inc_entry_api` (empty cache)",
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
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut group =
            c.benchmark_group("Compare: `remove` and `remove_entry_api` (empty cache)");
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
