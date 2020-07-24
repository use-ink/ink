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

    pub fn lazy_load(root_key: Key) -> BinaryHeap<u32> {
        <BinaryHeap<u32> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key))
    }

    pub fn push(mut heap: BinaryHeap<u32>, value: u32) {
        heap.push(value)
    }

    pub fn pop(mut heap: BinaryHeap<u32>) -> Option<u32> {
        heap.pop()
    }
}

// TODO: small steps - basic push/pop benches with varying sizes and ordering
// idea: add quickcheck tests for properties e.g. pop always greatest
// push should be average O(1), worst case O(log n)

fn bench_push_empty_cache(c: &mut Criterion) {
    bench_heap_sizes(
        c,
        "BinaryHeap::push (empty cache)",
        binary_heap::init_storage,
        Push { new_heap: NewHeap::Lazy }
    );
}

fn bench_push_populated_cache(c: &mut Criterion) {
    bench_heap_sizes(
        c,
        "BinaryHeap::push (populated cache)",
        |_: Key, _: &[u32]| {},
        Push { new_heap: NewHeap::Populated }
    );
}

fn bench_pop_empty_cache(c: &mut Criterion) {
    // let init = binary_heap::init_storage;
    // let setup = |root_key: Key, _: &[u32]| binary_heap::lazy_load(root_key);
    //
    // bench_heap_sizes(c, "BinaryHeap::pop (empty cache)", init, setup, binary_heap::pop);
}

fn bench_pop_populated_cache(c: &mut Criterion) {
    // let init = |_: Key, _: &[u32]| {};
    // let setup = |_: Key, test_values: &[u32]| binary_heap_from_slice(test_values);
    //
    // bench_heap_sizes(c, "BinaryHeap::push (populated cache)", init, setup, binary_heap::pop)
}

enum NewHeap {
    Lazy,
    Populated,
}

trait Benchmark {
    fn bench(&self, group: &mut BenchmarkGroup<WallTime>, size: u32, root_key: Key, values: &[u32]);
}

struct Push {
    new_heap: NewHeap,
}

impl Benchmark for Push {
    fn bench(&self, group: &mut BenchmarkGroup<WallTime>, size: u32, root_key: Key, values: &[u32]) {
        let setup = || match self.new_heap {
            NewHeap::Lazy => binary_heap::lazy_load(root_key),
            NewHeap::Populated => binary_heap::from_slice(values),
        };

        let largest_value = size + 1;
        group.bench_with_input(
            BenchmarkId::new("largest value", size),
            &largest_value,
            |b, &value| {
                b.iter_batched(
                    setup,
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
                    setup,
                    |mut heap| heap.push(value),
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn bench_heap_sizes<I, B>(c: &mut Criterion, name: &str, init: I, benchmark: B)
where
    I: FnOnce(Key, &[u32]) + Copy,
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

            benchmark.bench(&mut group, *size, root_key, &test_values)
        }

        group.finish();
        Ok(())
    })
    .unwrap();
}
