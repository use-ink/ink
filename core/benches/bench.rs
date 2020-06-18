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
    criterion_group,
    criterion_main,
    BenchmarkId,
    Criterion,
};

use core::mem::ManuallyDrop;
use ink_core::{
    env,
    storage2::{
        collections::Vec as StorageVec,
        traits::{
            KeyPtr,
            SpreadLayout,
        },
    },
};
use ink_primitives::Key;
use criterion::black_box;

criterion_group!(benches_clear_cached, bench_clear_cached);
criterion_group!(benches_clear_lazy, bench_clear_lazy);
criterion_group!(benches_put_cached, bench_put_cached);
criterion_group!(benches_put_lazy, bench_put_lazy);
criterion_main!(
    benches_clear_cached,
    benches_clear_lazy,
    benches_put_cached,
    benches_put_lazy
);

/// Asserts that the the given ordered storage vector elements are equal to the
/// ordered elements of the given slice.
fn assert_eq_slice(vec: &StorageVec<u8>, slice: &[u8]) {
    assert_eq!(vec.len() as usize, slice.len());
    assert!(vec.iter().zip(slice.iter()).all(|(lhs, rhs)| *lhs == *rhs))
}

/// Creates a storage vector from the given slice.
fn vec_from_slice(slice: &[u8]) -> StorageVec<u8> {
    slice.iter().copied().collect::<StorageVec<u8>>()
}

fn clear(test_values: &[u8]) {
    let mut vec = vec_from_slice(&test_values);
    black_box(vec.clear());
    assert!(vec.is_empty());
}

fn pop_all(test_values: &[u8]) {
    let mut vec = vec_from_slice(&test_values);
    while let Some(_) = black_box(vec.pop()) {}
    assert!(vec.is_empty());
}

fn put_cached(test_values: &[u8]) {
    let mut vec = vec_from_slice(&test_values);
    for (index, _value) in test_values.iter().enumerate() {
        vec.set(index as u32, b'X');
    }
    assert_eq_slice(&vec, &[b'X', b'X', b'X', b'X', b'X', b'X']);
}

fn deref_cached(test_values: &[u8]) {
    let mut vec = vec_from_slice(&test_values);
    for (index, _value) in test_values.iter().enumerate() {
        *vec.get_mut(index as u32).unwrap() = b'X';
    }
    assert_eq_slice(&vec, &[b'X', b'X', b'X', b'X', b'X', b'X']);
}

fn bench_put_cached(c: &mut Criterion) {
    let mut group = c.benchmark_group("PutMustOutperformDerefInCachedCase");
    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    group.bench_with_input(BenchmarkId::new("Put", 0), &test_values, |b, i| {
        b.iter(|| put_cached(i))
    });
    group.bench_with_input(BenchmarkId::new("Deref", 0), &test_values, |b, i| {
        b.iter(|| deref_cached(i))
    });
    group.finish();
}

fn bench_clear_cached(c: &mut Criterion) {
    let mut group = c.benchmark_group("ClearMustOutperformPopInCachedCase");
    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    group.bench_with_input(BenchmarkId::new("Clear", 0), &test_values, |b, i| {
        b.iter(|| clear(i))
    });
    group.bench_with_input(BenchmarkId::new("PopAll", 0), &test_values, |b, i| {
        b.iter(|| pop_all(i))
    });
    group.finish();
}

/// The manual drop is used to prevent the `vec` from being written back to storage.
/// This is so that it can be reused in the next benchmark instance, without the
/// storage flush overhead.
fn manual_drop(vec: StorageVec<u8>) {
    ManuallyDrop::new(vec);
}

/// In this case we lazily load the vec from storage using `pull_spread`.
/// This will just load lazily and won't pull anything from the storage.
fn clear_lazy() {
    let root_key = Key::from([0x00; 32]);
    let mut vec =
        <StorageVec<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
    black_box(vec.clear());
    assert!(vec.is_empty());
    manual_drop(vec);
}

/// In this case we lazily load the vec from storage using `pull_spread`.
/// This will just load lazily and won't pull anything from the storage.
/// `pop` will then result in loading from storage.
fn pop_all_lazy() {
    let root_key = Key::from([0x00; 32]);
    let mut vec =
        <StorageVec<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
    while let Some(_) = black_box(vec.pop()) {}
    assert!(vec.is_empty());
    manual_drop(vec);
}

/// In this case we lazily load the vec from storage using `pull_spread`.
/// This will just load lazily and won't pull anything from the storage.
/// The `deref` will then load from storage.
fn deref_lazy() {
    let root_key = Key::from([0x00; 32]);
    let mut vec =
        <StorageVec<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
    for index in 0..vec.len() {
        *vec.get_mut(index).unwrap() = b'X';
    }
    assert_eq_slice(&vec, &[b'X', b'X', b'X', b'X', b'X', b'X']);
    manual_drop(vec);
}

/// In this case we lazily load the vec from storage using `pull_spread`.
/// This will just load lazily and won't pull anything from the storage.
/// The `vec.set()` will not load anything from storage.
fn put_lazy() {
    let root_key = Key::from([0x00; 32]);
    let mut vec =
        <StorageVec<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
    for index in 0..vec.len() {
        vec.set(index, b'X');
    }
    assert_eq_slice(&vec, &[b'X', b'X', b'X', b'X', b'X', b'X']);
    manual_drop(vec);
}

/// Create a vec and push it to storage.
fn create_and_store() {
    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    let vec = vec_from_slice(&test_values);
    let root_key = Key::from([0x00; 32]);
    SpreadLayout::push_spread(&vec, &mut KeyPtr::from(root_key));
}

/// In this case we lazily instantiate a `StorageVec` by first `create_and_store`-ing
/// into the contract storage. We then load the vec from storage lazily in each
/// benchmark iteration.
fn bench_clear_lazy(c: &mut Criterion) {
    let mut group = c.benchmark_group("ClearMustOutperformPopInLazyCase");
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        create_and_store();
        group.bench_function(BenchmarkId::new("Clear", 0), |b| b.iter(|| clear_lazy()));
        Ok(())
    })
    .unwrap();
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        create_and_store();
        group
            .bench_function(BenchmarkId::new("PopAll", 0), |b| b.iter(|| pop_all_lazy()));
        Ok(())
    })
    .unwrap();
    group.finish();
}

/// In this case we lazily instantiate a `StorageVec` by first `create_and_store`-ing
/// into the contract storage. We then load the vec from storage lazily in each
/// benchmark iteration.
fn bench_put_lazy(c: &mut Criterion) {
    let mut group = c.benchmark_group("PutMustOutperformDerefInLazyCase");
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        create_and_store();
        group.bench_function(BenchmarkId::new("Put", 0), |b| b.iter(|| put_lazy()));
        Ok(())
    })
    .unwrap();
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        create_and_store();
        group.bench_function(BenchmarkId::new("Deref", 0), |b| b.iter(|| deref_lazy()));
        Ok(())
    })
    .unwrap();
    group.finish();
}
