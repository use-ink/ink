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
use ink_core::storage2::collections::Stash as StorageStash;

criterion_group!(benches, criterion_benchmark);
criterion_group!(benches_worst_case, criterion_benchmark_with_taken_value_read);
criterion_main!(benches, benches_worst_case);

fn remove_from_filled(test_values: &[u8; 6]) {
    let mut stash = test_values.iter().copied().collect::<StorageStash<_>>();

    for (index, _value) in test_values.iter().enumerate() {
        unsafe { stash.remove_occupied(index as u32) };
    }
    assert_eq!(stash.len(), 0);
}

fn take_from_filled(test_values: &[u8; 6]) {
    let mut stash = test_values.iter().copied().collect::<StorageStash<_>>();

    for (index, _value) in test_values.iter().enumerate() {
        stash.take(index as u32);
    }
    assert_eq!(stash.len(), 0);
}

fn take_from_filled_worst_case(test_values: &[u8; 6]) {
    let _ = env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
        // In order to enforce that the storage2::Stash actually loads the taken
        // value from the contract storage we have to instantiate an instance of
        // such a `test_values` storage Stash and then `push_spread` it onto the contract
        // storage using a known Key.
        let stash = test_values.iter().copied().collect::<StorageStash<_>>();
        let root_key = Key([0x42; 32]);
        SpreadLayout::push_spread(&stash, &mut KeyPtr::from(root_key));

        // When performing the benchmarks we then have to lazily instantiate such a
        // storage Stash using the `pull_spread` method using the same key. It will
        // just load lazily and won't pull anything from the storage, yet.
        // So using take from this instance will actually initiate loading from the
        // emulated contract storage.
        let mut stash =
            <StorageStash<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));

        for (index, _value) in test_values.iter().enumerate() {
            let _ = stash.take(index as u32);
        }
        assert_eq!(stash.len(), 0);

        Ok(())
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("RemoveMustOutperformTake");

    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    group.bench_with_input(BenchmarkId::new("Remove", 0), &test_values,
                           |b, i| b.iter(|| remove_from_filled(i)));
    group.bench_with_input(BenchmarkId::new("Take", 0), &test_values,
                           |b, i| b.iter(|| take_from_filled(i)));
    group.finish();
}

fn criterion_benchmark_with_taken_value_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("RemoveMustOutperformTakeWorstCase");

    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    group.bench_with_input(BenchmarkId::new("Remove", 0), &test_values,
                           |b, i| b.iter(|| remove_from_filled(i)));
    group.bench_with_input(BenchmarkId::new("TakeWorstCase", 0), &test_values,
                           |b, i| b.iter(|| take_from_filled_worst_case(i)));
    group.finish();
}
