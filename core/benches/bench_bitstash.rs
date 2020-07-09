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
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

use ink_core::{
    env,
    storage2::{
        collections::BitStash,
        traits::{
            KeyPtr,
            SpreadLayout,
        },
    },
};
use ink_primitives::Key;

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

/// Pulls a lazily loading `BitStash` instance from the contract storage.
fn pull_stash() -> BitStash {
    let root_key = Key::from([0x00; 32]);
    <BitStash as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key))
}

mod populated_cache {
    use super::*;

    pub fn fill_bitstash() {
        let mut stash = BitStash::default();
        for _ in 0..BENCH_ALLOCATIONS {
            black_box(stash.put());
        }
    }
}

fn bench_populated_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bench: `fill_bitstash` (populated cache)");
    group.bench_function("fill_bitstash", |b| {
        b.iter(|| populated_cache::fill_bitstash())
    });
    group.finish();
}

mod empty_cache {
    use super::*;

    /// In this case we lazily load the stash from storage using `pull_spread`.
    /// This will just load lazily and won't pull anything from the storage.
    /// `take` will then result in loading from storage.
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
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        let mut group = c.benchmark_group("Bench: `fill_bitstash` (empty cache)");
        group
            .bench_function("fill_bitstash", |b| b.iter(|| empty_cache::fill_bitstash()));
        group.finish();
        Ok(())
    })
    .unwrap();
}
