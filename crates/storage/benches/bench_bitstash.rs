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
    BatchSize,
    Criterion,
};

use ink_primitives::Key;
use ink_storage::{
    collections::BitStash,
    traits::{
        KeyPtr,
        SpreadLayout,
    },
};

const BENCH_ALLOCATIONS: u32 = 100_000;

criterion_group!(populated_cache, bench_populated_cache,);
criterion_group!(empty_cache, bench_empty_cache,);
criterion_main!(populated_cache, empty_cache,);

/// Creates a `BitStash` and pushes it to the contract storage.
fn push_stash() {
    let stash = BitStash::default();
    let root_key = Key::from([0x00; 32]);
    SpreadLayout::push_spread(&stash, &mut KeyPtr::from(root_key));
}

/// Creates a `BitStash` and pushes it to the contract storage.
fn push_stash_by_ref(stash: &BitStash) {
    let root_key = Key::from([0x00; 32]);
    SpreadLayout::push_spread(stash, &mut KeyPtr::from(root_key));
}

/// Pulls a lazily loading `BitStash` instance from the contract storage.
fn pull_stash() -> BitStash {
    let root_key = Key::from([0x00; 32]);
    <BitStash as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key))
}

/// Executes only a single `put` operation on the stash.
pub fn one_put(stash: &mut BitStash) {
    black_box(stash.put());
}

/// Returns a stash on which `100_000` `put` operations have been executed.
fn create_large_stash() -> BitStash {
    let mut stash = BitStash::default();
    for _ in 0..100_000 {
        stash.put();
    }
    stash
}

mod populated_cache {
    use super::*;

    /// Executes `put` operations on a new `BitStash` exactly `BENCH_ALLOCATIONS` times.
    pub fn fill_bitstash() {
        let mut stash = BitStash::default();
        for _ in 0..BENCH_ALLOCATIONS {
            black_box(stash.put());
        }
    }
}

fn bench_populated_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bench: populated cache");
    group.bench_function("fill_bitstash", |b| {
        b.iter(|| populated_cache::fill_bitstash())
    });
    group.bench_function("one_put", |b| {
        b.iter_batched_ref(
            || create_large_stash(),
            |stash| one_put(stash),
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

mod empty_cache {
    use super::*;

    /// Executes `put` operations on a new `BitStash` exactly `BENCH_ALLOCATIONS` times.
    pub fn fill_bitstash() {
        push_stash();
        let mut stash = pull_stash();
        for _ in 0..BENCH_ALLOCATIONS {
            black_box(stash.put());
        }
    }
}

/// In this case we lazily instantiate a `BitStash` by first creating and storing
/// into the contract storage. We then load the stash from storage lazily in each
/// benchmark iteration.
fn bench_empty_cache(c: &mut Criterion) {
    let _ = ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let mut group = c.benchmark_group("Bench: empty cache");
        group
            .bench_function("fill_bitstash", |b| b.iter(|| empty_cache::fill_bitstash()));
        group.bench_function("one_put", |b| {
            b.iter_batched_ref(
                || {
                    let stash = create_large_stash();
                    push_stash_by_ref(&stash);
                    pull_stash()
                },
                |stash| one_put(stash),
                BatchSize::SmallInput,
            )
        });
        group.finish();
        Ok(())
    })
    .unwrap();
}
