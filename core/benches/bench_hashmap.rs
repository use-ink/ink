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
criterion_group!(empty_cache, bench_insert, bench_remove,);
criterion_main!(populated_cache, empty_cache,);

/// Returns some test values for use in benchmarks.
fn test_values() -> Vec<(i32, i32)> {
    let mut v = Vec::new();
    for index in 0..500 {
        v.push((index, index));
    }
    v
}

/// Creates a `StorageHashMap` from the given slice.
fn hashmap_from_slice(slice: &[(i32, i32)]) -> StorageHashMap<i32, i32> {
    slice.iter().copied().collect::<StorageHashMap<i32, i32>>()
}

/// Returns always the same `KeyPtr`.
fn key_ptr() -> KeyPtr {
    let root_key = Key::from([0x42; 32]);
    KeyPtr::from(root_key)
}

/// Creates a `StorageHashMap` and pushes it to the contract storage.
fn push_storage_hashmap() {
    let test_values = test_values();
    let hmap = hashmap_from_slice(&test_values[..]);
    SpreadLayout::push_spread(&hmap, &mut key_ptr());
}

/// Pulls a lazily loading `StorageHashMap` instance from the contract storage.
fn pull_storage_hashmap() -> StorageHashMap<i32, i32> {
    <StorageHashMap<i32, i32> as SpreadLayout>::pull_spread(&mut key_ptr())
}

mod populated_cache {
    use super::*;

    pub fn insert_and_inc() {
        let test_values = test_values();
        let mut hmap = hashmap_from_slice(&test_values[..]);
        for key in 0..1000 {
            black_box({
                if !hmap.contains_key(&key) {
                    hmap.insert(key, key);
                }
                *hmap.get_mut(&key).unwrap() += 1;
            });
        }
    }

    pub fn insert_and_inc_entry_api() {
        let test_values = test_values();
        let mut hmap = hashmap_from_slice(&test_values[..]);
        for key in 0..1000 {
            black_box({
                let v = hmap.entry(key).or_insert(key);
                *v += 1;
            });
        }
    }

    pub fn remove() {
        let test_values = test_values();
        let mut hmap = hashmap_from_slice(&test_values[..]);
        for key in 0..1000 {
            black_box({
                if hmap.contains_key(&key) {
                    let _ = hmap.take(&key);
                }
            });
        }
    }

    pub fn remove_entry_api() {
        let test_values = test_values();
        let mut hmap = hashmap_from_slice(&test_values[..]);
        for key in 0..1000 {
            black_box({
                if let Entry::Occupied(o) = hmap.entry(key) {
                    o.remove();
                }
            });
        }
    }
}

fn bench_insert_populated_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group(
        "Compare: `insert_and_inc` and `insert_and_inc_entry_api` (populated cache)",
    );
    group.bench_function("insert_and_inc", |b| {
        b.iter(|| populated_cache::insert_and_inc())
    });
    group.bench_function("insert_and_inc_entry_api", |b| {
        b.iter(|| populated_cache::insert_and_inc_entry_api())
    });
    group.finish();
}

fn bench_remove_populated_cache(c: &mut Criterion) {
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut group = c.benchmark_group(
            "Compare: `remove` and `remove_entry_api` (populated cache)",
        );
        group.bench_function("remove", |b| b.iter(|| populated_cache::remove()));
        group.bench_function("remove_entry_api", |b| {
            b.iter(|| populated_cache::remove_entry_api())
        });
        group.finish();
        Ok(())
    })
    .unwrap();
}

mod empty_cache {
    use super::*;

    /// In this case we lazily load the map from storage using `pull_spread`.
    /// This will just load lazily and won't pull anything from the storage.
    pub fn insert_and_inc() {
        push_storage_hashmap();
        let mut hmap = pull_storage_hashmap();
        for key in 0..1000 {
            black_box({
                if !hmap.contains_key(&key) {
                    hmap.insert(key, key);
                }
                *hmap.get_mut(&key).unwrap() += 1;
            });
        }
    }

    /// In this case we lazily load the map from storage using `pull_spread`.
    /// This will just load lazily and won't pull anything from the storage.
    /// `take` will then result in loading from storage.
    pub fn insert_and_inc_entry_api() {
        push_storage_hashmap();
        let mut hmap = pull_storage_hashmap();
        for key in 0..1000 {
            black_box({
                let v = hmap.entry(key).or_insert(key);
                *v += 1;
            });
        }
    }

    pub fn remove() {
        let test_values = test_values();
        let mut hmap = hashmap_from_slice(&test_values[..]);
        for key in 0..500 {
            black_box({
                if hmap.contains_key(&key) {
                    let _ = hmap.take(&key);
                }
            });
        }
    }

    pub fn remove_entry_api() {
        let test_values = test_values();
        let mut hmap = hashmap_from_slice(&test_values[..]);
        for key in 0..500 {
            black_box({
                if let Entry::Occupied(o) = hmap.entry(key) {
                    o.remove();
                }
            });
        }
    }
}

fn bench_insert(c: &mut Criterion) {
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut group = c.benchmark_group(
            "Compare: `insert_and_inc` and `insert_and_inc_entry_api` (empty cache)",
        );
        group.bench_function("insert_and_inc", |b| {
            b.iter(|| empty_cache::insert_and_inc())
        });
        group.bench_function("insert_and_inc_entry_api", |b| {
            b.iter(|| empty_cache::insert_and_inc_entry_api())
        });
        group.finish();
        Ok(())
    })
    .unwrap();
}

fn bench_remove(c: &mut Criterion) {
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut group =
            c.benchmark_group("Compare: `remove` and `remove_entry_api` (empty cache)");
        group.bench_function("remove", |b| b.iter(|| empty_cache::remove()));
        group.bench_function("remove_entry_api", |b| {
            b.iter(|| empty_cache::remove_entry_api())
        });
        group.finish();
        Ok(())
    })
    .unwrap();
}
