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

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, BenchmarkGroup};
use ink_core::{
    env,
    storage2::{
        collections::BinaryHeap,
        traits::{
            KeyPtr,
            SpreadLayout,
        },
    },
};
use ink_primitives::Key;
use std::time::Duration;
use criterion::measurement::WallTime;

criterion_group!(push, bench_push_empty_cache, bench_push_populated_cache);
criterion_group!(pop, bench_pop_empty_cache, bench_pop_populated_cache);
criterion_main!(push, pop);

/// Returns some test values for use in benchmarks.
fn test_values(n: u32) -> Vec<u32> {
    std::iter::repeat(0u32)
        .take(n as usize)
        .enumerate()
        .map(|(i, _)| i as u32 + 1)
        .collect()
}

mod binary_heap {
    use super::*;

    pub fn init_storage(root_key: Key, values: &[u32]) {
        let heap = from_slice(values);
        SpreadLayout::push_spread(&heap, &mut KeyPtr::from(root_key));

        // prevents storage for the test heap being cleared when the heap is dropped after each
        // benchmark iteration
        env::test::set_clear_storage_disabled(true);
    }

    /// Creates a binary heap from the given slice.
    pub fn from_slice(slice: &[u32]) -> BinaryHeap<u32> {
        slice.iter().copied().collect::<BinaryHeap<u32>>()
    }
}

// TODO: small steps - basic push/pop benches with varying sizes and ordering
// idea: add quickcheck tests for properties e.g. pop always greatest
// push should be average O(1), worst case O(log n)

fn bench_push_empty_cache(c: &mut Criterion) {
    bench_heap_sizes::<_, _, Push>(
        c,
        "BinaryHeap::push (empty cache)",
        binary_heap::init_storage,
        NewHeap::lazy
    );
}

fn bench_push_populated_cache(c: &mut Criterion) {
    bench_heap_sizes::<_, _, Push>(
        c,
        "BinaryHeap::push (populated cache)",
        |_: Key, _: &[u32]| {},
        NewHeap::populated
    );
}

fn bench_pop_empty_cache(c: &mut Criterion) {
    bench_heap_sizes::<_, _, Pop>(
        c,
        "BinaryHeap::pop (empty cache)",
        binary_heap::init_storage,
        NewHeap::lazy
    );
}

fn bench_pop_populated_cache(c: &mut Criterion) {
    bench_heap_sizes::<_, _, Pop>(
        c,
        "BinaryHeap::pop (populated cache)",
        |_: Key, _: &[u32]| {},
        NewHeap::populated
    );
}

fn bench_heap_sizes<I, H, B>(c: &mut Criterion, name: &str, init: I, new_test_heap: H)
where
    I: Fn(Key, &[u32]),
    H: Fn(Key, Vec<u32>) -> NewHeap,
    B: Benchmark,
{
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut group = c.benchmark_group(name);
        group.warm_up_time(Duration::from_secs(6));
        group.measurement_time(Duration::from_secs(10));

        for (key, size) in [(0u8, 8u32), (1, 16), (2, 32), (3, 64)].iter() {
            let root_key = Key::from([*key; 32]);
            let test_values = test_values(*size);

            // perform one time initialization for this heap size
            init(root_key, &test_values);

            let test_heap = new_test_heap(root_key, test_values);
            <B as Benchmark>::bench(&mut group, *size, test_heap)
        }

        group.finish();
        Ok(())
    })
    .unwrap();
}

enum NewHeap {
    Lazy(Key),
    Populated(Vec<u32>),
}

impl NewHeap {
    pub fn lazy(key: Key, _values: Vec<u32>) -> Self {
        Self::Lazy(key)
    }

    pub fn populated(_key: Key, values: Vec<u32>) -> Self {
        Self::Populated(values)
    }

    pub fn create_heap(&self) -> BinaryHeap<u32> {
        match self {
            NewHeap::Lazy(root_key) => {
                <BinaryHeap<u32> as SpreadLayout>::pull_spread(&mut KeyPtr::from(*root_key))
            },
            NewHeap::Populated(ref values) => binary_heap::from_slice(values),
        }
    }
}

trait Benchmark {
    fn bench(group: &mut BenchmarkGroup<WallTime>, size: u32, new_heap: NewHeap);
}

/// Benchmark [`BinaryHeap::push`]
enum Push {}

impl Benchmark for Push {
    fn bench(group: &mut BenchmarkGroup<WallTime>, size: u32, new_heap: NewHeap) {
        let largest_value = size + 1;
        group.bench_with_input(
            BenchmarkId::new("largest value", size),
            &largest_value,
            |b, &value| {
                b.iter_batched(
                    || new_heap.create_heap(),
                    |mut heap| heap.push(value),
                    BatchSize::SmallInput,
                );
            },
        );

        let smallest_value = 0;
        group.bench_with_input(
            BenchmarkId::new("smallest value", size),
            &smallest_value,
            |b, &value| {
                b.iter_batched(
                    || new_heap.create_heap(),
                    |mut heap| heap.push(value),
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

/// Benchmark [`BinaryHeap::pop`]
enum Pop {}

impl Benchmark for Pop {
    fn bench(group: &mut BenchmarkGroup<WallTime>, size: u32, new_heap: NewHeap) {
        group.bench_function(
            BenchmarkId::new("largest value", size),
            |b| {
                b.iter_batched(
                    || new_heap.create_heap(),
                    |mut heap| heap.pop(),
                    BatchSize::SmallInput,
                );
            },
        );
    }
}
