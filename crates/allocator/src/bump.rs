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

//! A simple bump allocator.
//!
//! Its goal to have a much smaller footprint than the admittedly more full-featured `wee_alloc`
//! allocator which is currently being used by ink! smart contracts.
//!
//! The heap which will be used by this allocator is a single page of memory, which in Wasm is
//! 64KiB. We do not expect contracts to use more memory than this (for now), so we will throw an
//! OOM error instead of requesting more memory.

use core::alloc::{
    GlobalAlloc,
    Layout,
};

/// A page in Wasm is 64KiB
const PAGE_SIZE: usize = 64 * 1024;

static mut INNER: InnerAlloc = InnerAlloc::new();

/// A bump allocator suitable for use in a Wasm environment.
pub struct BumpAllocator;

impl BumpAllocator {
    /// Initialize the backing heap of the bump allocator.
    ///
    /// This function must only be called **once**, and it **must** be called before any
    /// allocations are made.
    #[inline]
    pub fn init(&self) {
        // SAFETY: We are in a single threaded context, so we don't have to worry about this being
        // concurrently mutated by multiple threads.
        unsafe { INNER.init() }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        INNER.alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

struct InnerAlloc {
    /// Points to the start of the next available allocation.
    next: usize,

    /// The address of the upper limit of our heap.
    upper_limit: usize,
}

impl InnerAlloc {
    const fn new() -> Self {
        Self {
            next: 0,
            upper_limit: 0,
        }
    }

    /// Initialize the heap which backs the bump allocator.
    ///
    /// Our heap is a single page of Wasm memory (64KiB) and will not grow beyond that.
    ///
    /// Note that this function must be called before any allocations can take place, otherwise any
    /// attempts to perform an allocation will fail.
    fn init(&mut self) {
        let prev_page = core::arch::wasm32::memory_grow(0, 1);
        if prev_page == usize::MAX {
            todo!()
        }

        let start = match prev_page.checked_mul(PAGE_SIZE) {
            Some(s) => s,
            None => todo!(),
        };

        self.upper_limit = match start.checked_add(PAGE_SIZE) {
            Some(u) => u,
            None => todo!(),
        };

        self.next = start;
    }

    /// Note: This function assumes that the allocator has already been initialized properly (see
    /// [Self::init()].
    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let alloc_start = self.next;

        let aligned_layout = layout.pad_to_align();
        let alloc_end = match alloc_start.checked_add(aligned_layout.size()) {
            Some(end) => end,
            None => return core::ptr::null_mut(),
        };

        if alloc_end > self.upper_limit {
            return core::ptr::null_mut()
        }

        self.next = alloc_end;
        alloc_start as *mut u8
    }
}
