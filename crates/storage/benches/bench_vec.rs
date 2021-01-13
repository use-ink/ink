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
    BenchmarkId,
    Criterion,
};
use ink_primitives::Key;
use ink_storage::{
    collections::Vec as StorageVec,
    traits::{
        KeyPtr,
        SpreadLayout,
    },
};

criterion_group!(
    populated_cache,
    bench_clear_populated_cache,
    bench_put_populated_cache,
);
criterion_group!(empty_cache, bench_clear_empty_cache, bench_put_empty_cache,);
criterion_main!(populated_cache, empty_cache,);

/// Returns some test values for use in benchmarks.
#[rustfmt::skip]
fn test_values() -> &'static [u8] {
    &[
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
        b'A', b'B', b'C', b'D', b'E', b'F'
    ]
}

/// Creates a storage vector from the given slice.
fn storage_vec_from_slice(slice: &[u8]) -> StorageVec<u8> {
    slice.iter().copied().collect::<StorageVec<u8>>()
}

/// Creates a storage vector and pushes it to the contract storage.
fn push_storage_vec() {
    let vec = storage_vec_from_slice(test_values());
    let root_key = Key::from([0x00; 32]);
    SpreadLayout::push_spread(&vec, &mut KeyPtr::from(root_key));
}

/// Pulls a lazily loading storage vector instance from the contract storage.
fn pull_storage_vec() -> StorageVec<u8> {
    let root_key = Key::from([0x00; 32]);
    <StorageVec<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key))
}

mod populated_cache {
    use super::*;

    pub fn clear(test_values: &[u8]) {
        let mut vec = storage_vec_from_slice(&test_values);
        black_box(vec.clear());
    }

    pub fn pop_all(test_values: &[u8]) {
        let mut vec = storage_vec_from_slice(&test_values);
        while let Some(ignored) = black_box(vec.pop()) {
            black_box(ignored);
        }
    }

    pub fn set(test_values: &[u8]) {
        let mut vec = storage_vec_from_slice(&test_values);
        for (index, _value) in test_values.iter().enumerate() {
            let _ = black_box(vec.set(index as u32, b'X'));
        }
    }

    pub fn get_mut(test_values: &[u8]) {
        let mut vec = storage_vec_from_slice(&test_values);
        for (index, _value) in test_values.iter().enumerate() {
            *black_box(vec.get_mut(index as u32).unwrap()) = b'X';
        }
    }
}

fn bench_put_populated_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("Compare: `set` and `get_mut` (populated cache)");
    group.bench_with_input(BenchmarkId::new("set", 0), test_values(), |b, i| {
        b.iter(|| populated_cache::set(i))
    });
    group.bench_with_input(BenchmarkId::new("get_mut", 0), test_values(), |b, i| {
        b.iter(|| populated_cache::get_mut(i))
    });
    group.finish();
}

fn bench_clear_populated_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("Compare: `clear` and `pop_all` (populated cache)");
    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    group.bench_with_input("clear", &test_values, |b, i| {
        b.iter(|| populated_cache::clear(i))
    });
    group.bench_with_input("pop_all", &test_values, |b, i| {
        b.iter(|| populated_cache::pop_all(i))
    });
    group.finish();
}

mod empty_cache {
    use super::*;

    /// In this case we lazily load the vec from storage using `pull_spread`.
    /// This will just load lazily and won't pull anything from the storage.
    pub fn clear() {
        push_storage_vec();
        let mut vec = pull_storage_vec();
        black_box(vec.clear());
    }

    /// In this case we lazily load the vec from storage using `pull_spread`.
    /// This will just load lazily and won't pull anything from the storage.
    /// `pop` will then result in loading from storage.
    pub fn pop_all() {
        push_storage_vec();
        let mut vec = pull_storage_vec();
        while let Some(ignored) = black_box(vec.pop()) {
            black_box(ignored);
        }
    }

    /// In this case we lazily load the vec from storage using `pull_spread`.
    /// This will just load lazily and won't pull anything from the storage.
    /// The `deref` will then load from storage.
    pub fn get_mut() {
        push_storage_vec();
        let mut vec = pull_storage_vec();
        for index in 0..vec.len() {
            *black_box(vec.get_mut(index).unwrap()) = b'X';
        }
    }

    /// In this case we lazily load the vec from storage using `pull_spread`.
    /// This will just load lazily and won't pull anything from the storage.
    /// The `vec.set()` will not load anything from storage.
    pub fn set() {
        push_storage_vec();
        let mut vec = pull_storage_vec();
        for index in 0..vec.len() {
            let _ = black_box(vec.set(index, b'X'));
        }
    }
}

/// In this case we lazily instantiate a `StorageVec` by first `create_and_store`-ing
/// into the contract storage. We then load the vec from storage lazily in each
/// benchmark iteration.
fn bench_clear_empty_cache(c: &mut Criterion) {
    let _ = ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let mut group = c.benchmark_group("Compare: `clear` and `pop_all` (empty cache)");
        group.bench_function("clear", |b| b.iter(|| empty_cache::clear()));
        group.bench_function("pop_all", |b| b.iter(|| empty_cache::pop_all()));
        group.finish();
        Ok(())
    })
    .unwrap();
}

/// In this case we lazily instantiate a `StorageVec` by first `create_and_store`-ing
/// into the contract storage. We then load the vec from storage lazily in each
/// benchmark iteration.
fn bench_put_empty_cache(c: &mut Criterion) {
    let _ = ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let mut group = c.benchmark_group("Compare: `set` and `get_mut` (empty cache)");
        group.bench_function("set", |b| b.iter(|| empty_cache::set()));
        group.bench_function("get_mut", |b| b.iter(|| empty_cache::get_mut()));
        group.finish();
        Ok(())
    })
    .unwrap();
}
