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

use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};
use ink_primitives::Key;
use ink_storage::{
    collections::Stash as StorageStash,
    traits::{
        KeyPtr,
        SpreadLayout,
    },
};

criterion_group!(
    populated_cache,
    bench_remove_occupied_populated_cache,
    bench_drain_with_populated_cache
);
criterion_group!(
    empty_cache,
    bench_remove_occupied_empty_cache,
    bench_drain_with_empty_cache
);
criterion_main!(populated_cache, empty_cache,);

/// The number of test values used in these benchmarks.
const COUNT_TEST_VALUES: usize = 10_000;

/// Returns test values for use in benchmarks.
fn generate_test_values() -> Vec<u32> {
    std::iter::repeat(0u32)
        .take(COUNT_TEST_VALUES)
        .enumerate()
        .map(|(i, _)| i as u32 + 1)
        .collect()
}

/// Creates a storage stash from the given slice.
fn storage_stash_from_slice(slice: &[u32]) -> StorageStash<u32> {
    slice.iter().copied().collect::<StorageStash<u32>>()
}

/// Creates a storage stash and pushes it to the contract storage.
fn push_storage_stash() {
    let stash = storage_stash_from_slice(&generate_test_values()[..]);
    let root_key = Key::from([0x00; 32]);
    SpreadLayout::push_spread(&stash, &mut KeyPtr::from(root_key));
}

/// Pulls a lazily loading storage stash instance from the contract storage.
fn pull_storage_stash() -> StorageStash<u32> {
    let root_key = Key::from([0x00; 32]);
    <StorageStash<u32> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key))
}

mod populated_cache {
    use super::*;

    pub fn remove_occupied_all(test_values: &[u32]) {
        let mut stash = storage_stash_from_slice(test_values);
        for index in 0..stash.len() {
            black_box(unsafe { stash.remove_occupied(index) });
        }
    }

    pub fn drain_with(test_values: &[u32]) {
        let mut stash = storage_stash_from_slice(test_values);
        black_box(stash.drain_with(|_| {}));
    }

    pub fn take_all(test_values: &[u32]) {
        let mut stash = storage_stash_from_slice(test_values);
        for (index, _) in test_values.iter().enumerate() {
            let _ = black_box(stash.take(index as u32));
        }
    }
}

fn bench_remove_occupied_populated_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group(
        "Compare: `remove_occupied_all` and `take_all` (populated cache)",
    );
    let test_values = &generate_test_values()[..];
    group.bench_with_input("remove_occupied_all", &test_values, |b, i| {
        b.iter(|| populated_cache::remove_occupied_all(i))
    });
    group.bench_with_input("take_all", &test_values, |b, i| {
        b.iter(|| populated_cache::take_all(i))
    });
    group.finish();
}

fn bench_drain_with_populated_cache(c: &mut Criterion) {
    let mut group =
        c.benchmark_group("Compare: `drain_with` and `take_all` (populated cache)");
    let test_values = &generate_test_values()[..];
    group.bench_with_input("drain_with", &test_values, |b, i| {
        b.iter(|| populated_cache::drain_with(i))
    });
    group.bench_with_input("take_all", &test_values, |b, i| {
        b.iter(|| populated_cache::take_all(i))
    });
    group.finish();
}

mod empty_cache {
    use super::*;

    /// In this case we lazily load the stash from storage using `pull_spread`.
    /// This will just load lazily and won't pull anything from the storage.
    pub fn remove_occupied_all() {
        push_storage_stash();
        let mut stash = pull_storage_stash();
        for index in 0..stash.len() {
            black_box(unsafe { stash.remove_occupied(index) });
        }
    }

    /// In this case we lazily load the stash from storage using `pull_spread`.
    /// This will just load lazily and won't pull anything from the storage.
    pub fn drain_with() {
        push_storage_stash();
        let mut stash = pull_storage_stash();
        black_box(stash.drain_with(|_| {}));
    }

    /// In this case we lazily load the stash from storage using `pull_spread`.
    /// This will just load lazily and won't pull anything from the storage.
    pub fn take_all() {
        push_storage_stash();
        let mut stash = pull_storage_stash();
        for index in 0..stash.len() {
            let _ = black_box(stash.take(index as u32));
        }
    }
}

/// In this case we lazily instantiate a `StorageStash` by first `create_and_store`-ing
/// into the contract storage. We then load the stash from storage lazily in each
/// benchmark iteration.
fn bench_remove_occupied_empty_cache(c: &mut Criterion) {
    let _ = ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let mut group = c.benchmark_group(
            "Compare: `remove_occupied_all` and `take_all` (empty cache)",
        );
        group.bench_function("remove_occupied_all", |b| {
            b.iter(|| empty_cache::remove_occupied_all())
        });
        group.bench_function("take_all", |b| b.iter(|| empty_cache::take_all()));
        group.finish();
        Ok(())
    })
    .unwrap();
}

/// In this case we lazily instantiate a `StorageStash` by first `create_and_store`-ing
/// into the contract storage. We then load the stash from storage lazily in each
/// benchmark iteration.
fn bench_drain_with_empty_cache(c: &mut Criterion) {
    let _ = ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let mut group =
            c.benchmark_group("Compare: `drain_with` and `take_all` (empty cache)");
        group.bench_function("drain_with", |b| b.iter(|| empty_cache::drain_with()));
        group.bench_function("take_all", |b| b.iter(|| empty_cache::take_all()));
        group.finish();
        Ok(())
    })
    .unwrap();
}
