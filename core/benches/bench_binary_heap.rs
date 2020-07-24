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
    BatchSize,
    BenchmarkId,
    Criterion,
};
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

criterion_group!(push, bench_push_empty_cache, bench_push_populated_cache);
criterion_main!(push,);

/// Returns some test values for use in benchmarks.
fn test_values(n: u32) -> Vec<u32> {
    std::iter::repeat(0u32)
        .take(n as usize)
        .enumerate()
        .map(|(i, _)| i as u32 + 1)
        .collect()
}

/// Creates a binary heap from the given slice.
fn binary_heap_from_slice(slice: &[u32]) -> BinaryHeap<u32> {
    slice.iter().copied().collect::<BinaryHeap<u32>>()
}

// TODO: small steps - basic push/pop benches with varying sizes and ordering
// idea: add quickcheck tests for properties e.g. pop always greatest
// push should be average O(1), worst case O(log n)

fn bench_push_empty_cache(c: &mut Criterion) {
    let init = |root_key: Key, test_values: &[u32]| {
        let heap = binary_heap_from_slice(test_values);
        SpreadLayout::push_spread(&heap, &mut KeyPtr::from(root_key));

        // prevents storage for the test heap being cleared when the heap is dropped after each
        // benchmark iteration
        env::test::set_clear_storage_disabled(true);
    };

    let setup = |root_key: Key, _: &[u32]| {
        <BinaryHeap<u32> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key))
    };

    bench_push(c, "BinaryHeap::push (empty cache)", init, setup);
}

fn bench_push_populated_cache(c: &mut Criterion) {
    let init = |_: Key, _: &[u32]| {};
    let setup = |_: Key, test_values: &[u32]| binary_heap_from_slice(test_values);

    bench_push(c, "BinaryHeap::push (populated cache)", init, setup)
}

fn bench_push<I, S>(c: &mut Criterion, name: &str, init: I, mut setup: S)
where
    I: FnOnce(Key, &[u32]) + Copy,
    S: FnMut(Key, &[u32]) -> BinaryHeap<u32>,
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

            let largest_value = size + 1;
            // bench_push(&mut group, "largest value", *size, largest_value, setup);
            group.bench_with_input(
                BenchmarkId::new("largest value", *size),
                &largest_value,
                |b, &value| {
                    b.iter_batched(
                        || setup(root_key, &test_values),
                        |mut heap| heap.push(value),
                        BatchSize::SmallInput,
                    );
                },
            );

            let smallest_value = 0;
            group.bench_with_input(
                BenchmarkId::new("smallest value", *size),
                &smallest_value,
                |b, &value| {
                    b.iter_batched(
                        || setup(root_key, &test_values),
                        |mut heap| heap.push(value),
                        BatchSize::SmallInput,
                    );
                },
            );
        }

        group.finish();
        Ok(())
    })
    .unwrap();
}
