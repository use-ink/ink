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

criterion_group!(populated_cache, bench_remove_occupied_populated_cache,);
criterion_group!(empty_cache, bench_remove_occupied_empty_cache,);
criterion_main!(populated_cache, empty_cache,);

/// Returns some test values for use in benchmarks.
#[rustfmt::skip]
fn test_values() -> &'static [u8] {
    &[
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
        b'A', b'B', b'C', b'D', b'E', b'F'
    ]
}

/// Creates a storage stash from the given slice.
fn storage_stash_from_slice(slice: &[u8]) -> StorageStash<u8> {
    slice.iter().copied().collect::<StorageStash<u8>>()
}

/// Creates a storage stash and pushes it to the contract storage.
fn push_storage_stash() {
    let stash = storage_stash_from_slice(test_values());
    let root_key = Key::from([0x00; 32]);
    SpreadLayout::push_spread(&stash, &mut KeyPtr::from(root_key));
}

/// Pulls a lazily loading storage stash instance from the contract storage.
fn pull_storage_stash() -> StorageStash<u8> {
    let root_key = Key::from([0x00; 32]);
    <StorageStash<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key))
}

mod populated_cache {
    use super::*;

    pub fn remove_occupied_all(test_values: &[u8]) {
        let mut stash = storage_stash_from_slice(test_values);
        for (index, _value) in test_values.iter().enumerate() {
            black_box(unsafe { stash.remove_occupied(index as u32) });
        }
    }

    pub fn take_all(test_values: &[u8]) {
        let mut stash = storage_stash_from_slice(test_values);
        for (index, _value) in test_values.iter().enumerate() {
            black_box(stash.take(index as u32));
        }
    }
}

fn bench_remove_occupied_populated_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group(
        "Compare: `remove_occupied_all` and `take_all` (populated cache)",
    );
    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    group.bench_with_input("remove_occupied_all", &test_values, |b, i| {
        b.iter(|| populated_cache::remove_occupied_all(i))
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
    /// `take` will then result in loading from storage.
    pub fn take_all() {
        push_storage_stash();
        let mut stash = pull_storage_stash();
        for index in 0..stash.len() {
            black_box(stash.take(index));
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
