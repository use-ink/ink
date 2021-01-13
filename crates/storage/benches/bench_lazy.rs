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
    lazy::Lazy,
    traits::{
        KeyPtr,
        SpreadLayout,
    },
};

criterion_group!(populated_cache, bench_set_populated_cache);
criterion_group!(empty_cache, bench_set_empty_cache,);
criterion_main!(populated_cache, empty_cache);

mod populated_cache {
    use super::*;
    use core::ops::DerefMut;

    pub fn set() {
        let mut lazy = <Lazy<i32>>::new(1);
        let lazy_mut = black_box(&mut lazy);
        black_box(Lazy::set(lazy_mut, 17));
    }

    pub fn deref_mut() {
        let mut lazy = <Lazy<i32>>::new(1);
        let i32_mut = black_box(lazy.deref_mut());
        black_box(*i32_mut = 17);
    }
}

fn bench_set_populated_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("Compare: `set` and `deref_mut` (populated cache)");
    group.bench_function(BenchmarkId::new("set", 0), |b| {
        b.iter(|| populated_cache::set())
    });
    group.bench_function(BenchmarkId::new("deref_mut", 0), |b| {
        b.iter(|| populated_cache::deref_mut())
    });
    group.finish();
}

/// Pushes a value to contract storage and creates a `Lazy` pointing to it.
fn push_storage_lazy(value: i32) -> Lazy<i32> {
    let root_key = Key::from([0x00; 32]);
    SpreadLayout::push_spread(&Lazy::new(value), &mut KeyPtr::from(root_key));
    SpreadLayout::pull_spread(&mut KeyPtr::from(root_key))
}

mod empty_cache {
    use super::*;

    pub fn set() {
        let mut lazy = push_storage_lazy(1);
        black_box(Lazy::set(&mut lazy, 13));
    }

    pub fn deref_mut() {
        let mut lazy = push_storage_lazy(1);
        black_box(*lazy = 13);
    }
}

fn bench_set_empty_cache(c: &mut Criterion) {
    let _ = ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
        let mut group = c.benchmark_group("Compare: `set` and `deref_mut` (empty cache)");
        group.bench_function(BenchmarkId::new("set", 0), |b| {
            b.iter(|| empty_cache::set())
        });
        group.bench_function(BenchmarkId::new("deref_mut", 0), |b| {
            b.iter(|| empty_cache::deref_mut())
        });
        group.finish();
        Ok(())
    })
    .unwrap();
}
