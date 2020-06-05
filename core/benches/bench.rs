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

use ink_core::storage2::collections::Stash as StorageStash;

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn remove_from_filled(test_values: &[u8; 6]) {
    let mut stash = test_values.iter().copied().collect::<StorageStash<_>>();

    for (index, _value) in test_values.iter().enumerate() {
        stash.remove(index as u32);
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

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("RemoveMustOutperformTake");

    let test_values = [b'A', b'B', b'C', b'D', b'E', b'F'];
    group.bench_with_input(BenchmarkId::new("Remove", 0), &test_values,
                           |b, i| b.iter(|| remove_from_filled(i)));
    group.bench_with_input(BenchmarkId::new("Take", 0), &test_values,
                           |b, i| b.iter(|| take_from_filled(i)));
    group.finish();
}
