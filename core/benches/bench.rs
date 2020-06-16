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

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use ink_core::{
    env,
    storage2::traits::{
        KeyPtr,
        SpreadLayout,
    },
};
use ink_primitives::Key;
use ink_core::storage2::collections::Vec as StorageVec;

criterion_group!(benches, criterion_clear);
criterion_group!(benches_put, criterion_put);
criterion_main!(benches, benches_put);

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
    assert_eq!(vec.len(), 0);
}

fn pop_all(test_values: &[u8]) {
    let mut vec = vec_from_slice(&test_values);

    while vec.len() > 0 {
        vec.pop();
    }
    assert_eq!(vec.len(), 0);
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

    let mut vec = vec_from_slice(&test_values);
    for (index, _value) in test_values.iter().enumerate() {
        *vec.get_mut(index as u32).unwrap() = b'X';
    }
    assert_eq_slice(&vec, &[b'X', b'X', b'X', b'X', b'X', b'X']);
}

fn criterion_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("ClearMustOutperformIterativePop");

    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    group.bench_with_input(BenchmarkId::new("Clear", 0), &test_values,
                           |b, i| b.iter(|| clear(i)));
    group.bench_with_input(BenchmarkId::new("PopAll", 0), &test_values,
                           |b, i| b.iter(|| pop_all(i)));
    group.finish();
}

fn criterion_put(c: &mut Criterion) {
    let mut group = c.benchmark_group("PutMustOutperformDeref");

    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    group.bench_with_input(BenchmarkId::new("Put", 0), &test_values,
                           |b, i| b.iter(|| put(i)));
    group.bench_with_input(BenchmarkId::new("Deref", 0), &test_values,
                           |b, i| b.iter(|| deref(i)));
    group.finish();
}
