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
    vec.clear();
    assert!(vec.is_empty());
}

fn pop_all(test_values: &[u8]) {
    let mut vec = vec_from_slice(&test_values);
    while let Some(_) = vec.pop() {}
    assert!(vec.is_empty());
}

/// In this case we lazily instantiate a `StorageVec` by first `push_spread`-ing
/// onto the contract storage. We then load the vec from storage lazily using
/// `pull_spread`. This will just load lazily and won't pull anything from the
/// storage.
fn clear_lazy(test_values: &[u8]) {
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let vec = vec_from_slice(&test_values);
        let root_key = Key::from([0x00; 32]);
        SpreadLayout::push_spread(&vec, &mut KeyPtr::from(root_key));
        let mut vec =
            <StorageVec<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        vec.clear();
        assert!(vec.is_empty());
        Ok(())
    })
    .unwrap();
}

/// In this case we lazily instantiate a `StorageVec` by first `push_spread`-ing
/// onto the contract storage. We then load the vec from storage lazily using
/// `pull_spread`. This will just load lazily and won't pull anything from the
/// storage. `pop` will then result in loading from storage.
fn pop_all_lazy(test_values: &[u8]) {
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let vec = vec_from_slice(&test_values);
        let root_key = Key::from([0x00; 32]);
        SpreadLayout::push_spread(&vec, &mut KeyPtr::from(root_key));
        let mut vec =
            <StorageVec<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        while let Some(_) = vec.pop() {}
        assert!(vec.is_empty());
        Ok(())
    })
    .unwrap();
}

fn put(test_values: &[u8]) {
    let mut vec = vec_from_slice(&test_values);
    for (index, _value) in test_values.iter().enumerate() {
        vec.set(index as u32, b'X');
    }
    assert_eq_slice(&vec, &[b'X', b'X', b'X', b'X', b'X', b'X']);
}

fn deref(test_values: &[u8]) {
    let mut vec = vec_from_slice(&test_values);
    for (index, _value) in test_values.iter().enumerate() {
        *vec.get_mut(index as u32).unwrap() = b'X';
    }
    assert_eq_slice(&vec, &[b'X', b'X', b'X', b'X', b'X', b'X']);
}

/// In this case we lazily instantiate a `StorageVec` by first `push_spread`-ing
/// onto the contract storage. We then load the vec from storage lazily using
/// `pull_spread`. This will just load lazily and won't pull anything from the
/// storage. The `vec.set()` will not load anything from storage.
fn put_lazy(test_values: &[u8]) {
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let vec = vec_from_slice(&test_values);
        let root_key = Key::from([0x00; 32]);
        SpreadLayout::push_spread(&vec, &mut KeyPtr::from(root_key));
        let mut vec =
            <StorageVec<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        for (index, _value) in test_values.iter().enumerate() {
            vec.set(index as u32, b'X');
        }
        assert_eq_slice(&vec, &[b'X', b'X', b'X', b'X', b'X', b'X']);
        Ok(())
    })
    .unwrap();
}

/// In this case we lazily instantiate a `StorageVec` by first `push_spread`-ing
/// onto the contract storage. We then load the vec from storage lazily using
/// `pull_spread`. This will just load lazily and won't pull anything from the
/// storage. The `deref` will then load from storage.
fn deref_lazy(test_values: &[u8]) {
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let vec = vec_from_slice(&test_values);
        let root_key = Key::from([0x00; 32]);
        SpreadLayout::push_spread(&vec, &mut KeyPtr::from(root_key));
        let mut vec =
            <StorageVec<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        for (index, _value) in test_values.iter().enumerate() {
            *vec.get_mut(index as u32).unwrap() = b'X';
        }
        assert_eq_slice(&vec, &[b'X', b'X', b'X', b'X', b'X', b'X']);
        Ok(())
    })
    .unwrap();
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

fn bench_clear_lazy(c: &mut Criterion) {
    let mut group = c.benchmark_group("ClearMustOutperformPopInLazyCase");

    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    group.bench_with_input(BenchmarkId::new("Clear", 0), &test_values, |b, i| {
        b.iter(|| clear_lazy(i))
    });
    group.bench_with_input(BenchmarkId::new("PopAll", 0), &test_values, |b, i| {
        b.iter(|| pop_all_lazy(i))
    });
    group.finish();
}

fn bench_put_cached(c: &mut Criterion) {
    let mut group = c.benchmark_group("PutMustOutperformDerefInCachedCase");

    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    group.bench_with_input(BenchmarkId::new("Put", 0), &test_values, |b, i| {
        b.iter(|| put(i))
    });
    group.bench_with_input(BenchmarkId::new("Deref", 0), &test_values, |b, i| {
        b.iter(|| deref(i))
    });
    group.finish();
}

fn bench_put_lazy(c: &mut Criterion) {
    let mut group = c.benchmark_group("PutMustOutperformDerefInLazyCase");

    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    group.bench_with_input(BenchmarkId::new("Put", 0), &test_values, |b, i| {
        b.iter(|| put_lazy(i))
    });
    group.bench_with_input(BenchmarkId::new("Deref", 0), &test_values, |b, i| {
        b.iter(|| deref_lazy(i))
    });
    group.finish();
}
