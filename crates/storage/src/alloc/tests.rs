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

use super::{
    alloc,
    free,
    ContractPhase,
    DynamicAllocation,
    DynamicAllocator,
};
use crate::{
    alloc,
    traits::{
        KeyPtr,
        SpreadLayout,
    },
};
use ink_env::{
    test,
    DefaultEnvironment,
};
use ink_primitives::Key;

fn run_default_test<F>(f: F)
where
    F: FnOnce(),
{
    alloc::initialize(ContractPhase::Deploy);
    test::run_test::<DefaultEnvironment, _>(|_| {
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
        assert_eq!(alloc(), DynamicAllocation(0));
    })
}

#[test]
fn alloc_free_in_the_middle() {
    run_default_test(|| {
        for i in 0..TEST_ALLOCATIONS {
            assert_eq!(alloc(), DynamicAllocation(i));
        }
        for i in 0..TEST_ALLOCATIONS {
            free(DynamicAllocation(i));
            assert_eq!(alloc(), DynamicAllocation(i));
        }
    })
}

#[test]
#[should_panic(expected = "encountered double free of dynamic storage: at index 0")]
fn double_free_panics() {
    run_default_test(|| {
        let a0 = alloc();
        let _ = alloc();
        free(a0);
        free(a0);
    })
}

#[test]
#[should_panic(expected = "invalid dynamic storage allocation")]
fn free_out_of_bounds() {
    run_default_test(|| {
        free(DynamicAllocation(0));
    })
}

fn spread_layout_alloc_setup() -> DynamicAllocator {
    let mut alloc = DynamicAllocator::default();
    assert_eq!(alloc.alloc(), DynamicAllocation(0));
    assert_eq!(alloc.alloc(), DynamicAllocation(1));
    assert_eq!(alloc.alloc(), DynamicAllocation(2));
    assert_eq!(alloc.alloc(), DynamicAllocation(3));
    assert_eq!(alloc.alloc(), DynamicAllocation(4));
    alloc.free(DynamicAllocation(3));
    alloc.free(DynamicAllocation(1));
    alloc
}

#[test]
fn spread_pull_push_works() {
    run_default_test(|| {
        let mut alloc = spread_layout_alloc_setup();
        let root_key = Key::from([0x77; 32]);
        // Push the current state of the dynamic storage allocator to the storage:
        SpreadLayout::push_spread(&alloc, &mut KeyPtr::from(root_key));
        // Now check if the new allocations are filling the freed ones:
        assert_eq!(alloc.alloc(), DynamicAllocation(1));
        assert_eq!(alloc.alloc(), DynamicAllocation(3));
        // Pull another instance of the storage allocator from storage,
        // then check if both allocators are equal after also allocating the same
        // allocation slots:
        let mut alloc2 =
            <DynamicAllocator as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(alloc2.alloc(), DynamicAllocation(1));
        assert_eq!(alloc2.alloc(), DynamicAllocation(3));
        assert_eq!(alloc2, alloc);
    })
}

#[test]
#[should_panic(expected = "encountered empty storage cell")]
fn spread_clear_works() {
    run_default_test(|| {
        let alloc = spread_layout_alloc_setup();
        let root_key = Key::from([0x42; 32]);
        // Push the current state of the dynamic storage allocator to the storage:
        SpreadLayout::push_spread(&alloc, &mut KeyPtr::from(root_key));
        // Pull another instance of the storage allocator from storage,
        // then check if both allocators are equal after also allocating the same
        // allocation slots:
        let alloc2 =
            <DynamicAllocator as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        assert_eq!(alloc2, alloc);
        // Now clear the storage associated with `alloc2` again and test if another
        // loaded instance from the same storage region panics upon pulling:
        SpreadLayout::clear_spread(&alloc2, &mut KeyPtr::from(root_key));
        // We have to prevent calling `Drop` of `alloc3` since it has been created
        // deliberately upon invalid contract storage. Since interacting with `alloc3`
        // panics which immediately initiates the dropping routines we have to
        // wrap it in `ManuallyDrop` before we interact with it to avoid to panic
        // while panicking.
        let alloc3 =
            <DynamicAllocator as SpreadLayout>::pull_spread(&mut KeyPtr::from(root_key));
        let mut alloc3 = core::mem::ManuallyDrop::new(alloc3);
        // Now interact with `alloc3` to make it load from the invalid storage:
        let _ = alloc3.alloc();
    })
}

#[test]
fn test_call_setup_works() {
    test::run_test::<DefaultEnvironment, _>(|_| {
        let mut allocator = DynamicAllocator::default();
        assert_eq!(allocator.alloc(), DynamicAllocation(0));
        assert_eq!(allocator.alloc(), DynamicAllocation(1));
        let root_key = Key::from([0xFE; 32]);
        DynamicAllocator::push_spread(&allocator, &mut KeyPtr::from(root_key));
        alloc::initialize(ContractPhase::Call);
        assert_eq!(alloc(), DynamicAllocation(2));
        assert_eq!(alloc(), DynamicAllocation(3));
        free(DynamicAllocation(0));
        free(DynamicAllocation(2));
        Ok(())
    })
    .unwrap();
}
