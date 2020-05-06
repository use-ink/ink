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

use super::{
    alloc,
    free,
    reset_allocator,
    set_contract_phase,
    ContractPhase,
    DynamicAllocation,
};
use crate::env::{
    test,
    DefaultEnvTypes,
};

fn run_default_test<F>(f: F)
where
    F: FnOnce(),
{
    set_contract_phase(ContractPhase::Deploy);
    reset_allocator();
    test::run_test::<DefaultEnvTypes, _>(|_| {
        f();
        Ok(())
    })
    .unwrap();
}

#[test]
fn alloc_works() {
    run_default_test(|| {
        assert_eq!(alloc(), DynamicAllocation(0));
    })
}

cfg_if::cfg_if! {
    if #[cfg(miri)] {
        // We need to lower the test allocations because miri's stacked borrows
        // analysis currently is super linear for some work loads.
        // Read more here: https://github.com/rust-lang/miri/issues/1367
        const TEST_ALLOCATIONS: u32 = 10;
    } else {
        const TEST_ALLOCATIONS: u32 = 10_000;
    }
}

#[test]
fn many_allocs_works() {
    run_default_test(|| {
        for i in 0..TEST_ALLOCATIONS {
            assert_eq!(alloc(), DynamicAllocation(i));
        }
    })
}

#[test]
fn free_works() {
    run_default_test(|| {
        // Check that this pattern does not panic.
        free(alloc());
    })
}

#[test]
fn many_alloc_and_free_works() {
    run_default_test(|| {
        for i in 0..TEST_ALLOCATIONS {
            assert_eq!(alloc(), DynamicAllocation(i));
        }
        for i in 0..TEST_ALLOCATIONS {
            free(DynamicAllocation(i))
        }
    })
}
