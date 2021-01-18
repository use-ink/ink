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
    Criterion,
};
use ink_primitives::{
    Key,
    KeyPtr,
};

criterion_group!(
    bench_key,
    bench_key_add_assign_u64,
    bench_key_add_assign_u64_one_ofvl,
    bench_key_add_assign_u64_two_ofvls,
    bench_key_add_assign_u64_three_ofvls,
    bench_key_add_assign_u64_wrap,
);
criterion_group!(
    bench_key_ptr,
    bench_key_ptr_advance_by,
    bench_key_ptr_advance_by_repeat,
);
criterion_main!(bench_key, bench_key_ptr);

fn bench_key_add_assign_u64(c: &mut Criterion) {
    let key = Key::from([0x00; 32]);
    c.bench_function("Key2::add_assign(u64)", |b| {
        b.iter(|| {
            let mut copy = black_box(key);
            let _ = black_box(copy += 1u64);
        })
    });
}

fn bench_key_add_assign_u64_one_ofvl(c: &mut Criterion) {
    let key = Key::from([
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    c.bench_function("Key2::add_assign(u64) - 1 ofvl", |b| {
        b.iter(|| {
            let mut copy = black_box(key);
            let _ = black_box(copy += 1u64);
        })
    });
}

fn bench_key_add_assign_u64_two_ofvls(c: &mut Criterion) {
    let key = Key::from([
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    c.bench_function("Key2::add_assign(u64) - 2 ofvls", |b| {
        b.iter(|| {
            let mut copy = black_box(key);
            let _ = black_box(copy += 1u64);
        })
    });
}

fn bench_key_add_assign_u64_three_ofvls(c: &mut Criterion) {
    let key = Key::from([
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    c.bench_function("Key2::add_assign(u64) - 3 ofvls", |b| {
        b.iter(|| {
            let mut copy = black_box(key);
            let _ = black_box(copy += 1u64);
        })
    });
}

fn bench_key_add_assign_u64_wrap(c: &mut Criterion) {
    let key = Key::from([0xFF; 32]);
    c.bench_function("Key2::add_assign(u64) - wrap", |b| {
        b.iter(|| {
            let mut copy = black_box(key);
            let _ = black_box(copy += 1u64);
        })
    });
}

fn bench_key_ptr_advance_by(c: &mut Criterion) {
    let key = Key::from([0x00; 32]);
    c.bench_function("KeyPtr2::advance_by copy", |b| {
        b.iter(|| {
            let mut key_ptr = KeyPtr::from(key.clone());
            let _ = black_box(key_ptr.advance_by(1));
        })
    });
}

fn bench_key_ptr_advance_by_repeat(c: &mut Criterion) {
    let key = Key::from([0x00; 32]);
    let mut key_ptr = KeyPtr::from(key.clone());
    c.bench_function("KeyPtr2::advance_by reuse", |b| {
        b.iter(|| {
            let _ = black_box(key_ptr.advance_by(1));
        })
    });
}
