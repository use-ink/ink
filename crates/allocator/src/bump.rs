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
//! It's goal to have a much smaller footprint than the admittedly more full-featured `wee_alloc`
//! allocator which is currently being used by ink! smart contracts.

use core::alloc::{GlobalAlloc, Layout};

/// A page in Wasm is 64KiB
const PAGE_SIZE: usize = 64 * 1024;

lazy_static::lazy_static! {
    pub static ref BUMP_ALLOC: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
}

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

pub struct BumpAllocator {
    /// Points to the start of the next available allocation
    next: usize,
    /// Hacks
    heap_initialized: bool,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            next: 0,
            heap_initialized: false,
        }
    }

    /// Initalize the backing heap of the allocator.
    //
    // In this case we'll be backed by a page of Wasm memory which is all we'll use for the life of
    // the contract.
    pub fn init(&mut self) {
        let ptr = core::arch::wasm32::memory_grow(0, 1);
        if ptr == usize::max_value() {
            todo!("TODO: OOM")
        }
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // This should be okay performance wise since we're in a single threaded context anyways
        let mut bump = self.lock();

        // TODO: Figure out how to properly initalize the heap
        if !bump.heap_initialized {
            let ptr = core::arch::wasm32::memory_grow(0, 1);
            if ptr == usize::max_value() {
                todo!("TODO: OOM")
            }
        }

        let aligned_layout = layout.pad_to_align();

        let alloc_start = bump.next;
        let alloc_end = match alloc_start.checked_add(aligned_layout.size()) {
            Some(end) => end,
            None => return core::ptr::null_mut(),
        };

        // Since we're using a single page as our entire heap if we exceed it we're effectively
        // out-of-memory.
        if alloc_end > PAGE_SIZE {
            return core::ptr::null_mut();
        }

        bump.next = alloc_end;
        alloc_start as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        todo!();
    }
}
