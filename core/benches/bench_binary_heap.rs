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
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut group = c.benchmark_group("BinaryHeap::push (empty cache)");

        for (key, size) in [(0u8, 8), (1, 16), (2, 32), (3, 64)].iter() {
            let test_values = test_values(*size);
            let heap = binary_heap_from_slice(&test_values);
            let root_key = Key::from([*key; 32]);
            SpreadLayout::push_spread(&heap, &mut KeyPtr::from(root_key));

            // prevents storage for the test heap being cleared when the heap is dropped after each
            // benchmark iteration
            env::test::set_clear_storage_disabled(true);

            let largest_value = size + 1;
            group.bench_with_input(
                BenchmarkId::new("largest value", size),
                &largest_value,
                |b, &value| {
                    b.iter_batched(
                        || {
                            <BinaryHeap<u32> as SpreadLayout>::pull_spread(
                                &mut KeyPtr::from(root_key),
                            )
                        },
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
                        || {
                            <BinaryHeap<u32> as SpreadLayout>::pull_spread(
                                &mut KeyPtr::from(root_key),
                            )
                        },
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

fn bench_push_populated_cache(c: &mut Criterion) {
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut group = c.benchmark_group("BinaryHeap::push (populated cache)");

        for size in [8, 16, 32, 64].iter() {
            let largest_value = size + 1;
            group.bench_with_input(
                BenchmarkId::new("largest value", size),
                &largest_value,
                |b, value| {
                    b.iter_batched(
                        || {
                            let test_values = test_values(*size);
                            binary_heap_from_slice(&test_values)
                        },
                        |mut heap| heap.push(*value),
                        BatchSize::SmallInput,
                    );
                },
            );

            let smallest_value = 0;
            group.bench_with_input(
                BenchmarkId::new("smallest value", size),
                &smallest_value,
                |b, value| {
                    b.iter_batched(
                        || {
                            let test_values = test_values(*size);
                            binary_heap_from_slice(&test_values)
                        },
                        |mut heap| heap.push(*value),
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
