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

unsafe impl GlobalAlloc for BumpAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match INNER.alloc(layout) {
            Some(start) => start as *mut u8,
            None => core::ptr::null_mut(),
        }
    }

    #[inline]
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg_attr(feature = "std", derive(Debug, Copy, Clone))]
struct InnerAlloc {
    /// Points to the start of the next available allocation.
    next: usize,

    /// The address of the upper limit of our heap.
    upper_limit: usize,

    /// How many page requests have we made?
    #[cfg(feature = "std")]
    page_requests: usize,
}

impl InnerAlloc {
    const fn new() -> Self {
        Self {
            next: 0,
            upper_limit: 0,
            #[cfg(feature = "std")]
            page_requests: 0,
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(all(not(feature = "std"), target_arch = "wasm32"))] {
            /// Request a new page of Wasm memory (64KiB).
            ///
            /// Returns `None` if a page isn't available.
            fn request_page(&mut self) -> Option<usize> {
                let prev_page = core::arch::wasm32::memory_grow(0, 1);
                if prev_page == usize::MAX {
                    return None;
                }

                prev_page.checked_mul(PAGE_SIZE)
            }

        } else if #[cfg(all(feature = "std", unix))] {
            /// Request a new Wasm page sized section (64KiB) of memory.
            ///
            /// Returns `None` if a page isn't available.
            ///
            /// This implementation is only meant to be used for testing, since we cannot (easily)
            /// test the `wasm32` implementation.
            fn request_page(&mut self) -> Option<usize> {
                let prev_page = self.page_requests.checked_mul(PAGE_SIZE);
                self.page_requests += 1;
                prev_page
            }
        } else {
            compile_error! {
                "ink! only supports compilation as `std` or `no_std` + `wasm32-unknown`"
            }
        }
    }

    /// Tries to allocate enough memory on the heap for the given `Layout`. If there isn't enough
    /// room on the heap it'll try and grow it by a page.
    // NOTE: I think we'll end up with fragmentation here
    fn alloc(&mut self, layout: Layout) -> Option<usize> {
        let alloc_start = self.next;

        let aligned_layout = dbg!(layout.pad_to_align());
        let alloc_end = alloc_start.checked_add(aligned_layout.size())?;

        if alloc_end > self.upper_limit {
            let page_start = self.request_page()?;
            self.upper_limit = page_start.checked_add(PAGE_SIZE)?;
            self.next = page_start.checked_add(aligned_layout.size())?;

            Some(page_start)
        } else {
            self.next = alloc_end;
            Some(alloc_start)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_alloc_a_byte() {
        let mut inner = InnerAlloc::new();

        let layout = Layout::new::<u8>();
        assert!(inner.alloc(layout).is_some());

        let expected_limit = inner.page_requests * PAGE_SIZE;
        assert_eq!(inner.upper_limit, expected_limit);

        let expected_alloc_start = 1 * std::mem::size_of::<u8>();
        assert_eq!(inner.next, expected_alloc_start);
    }

    #[test]
    fn can_alloc_a_foobarbaz() {
        let mut inner = InnerAlloc::new();

        struct FooBarBaz {
            _foo: u32,
            _bar: u128,
            _baz: (u16, bool),
        }

        let layout = Layout::new::<FooBarBaz>();

        let allocations = 3;
        for _ in 0..allocations {
            assert!(inner.alloc(layout).is_some());
        }

        let expected_limit = inner.page_requests * PAGE_SIZE;
        assert_eq!(inner.upper_limit, expected_limit);

        let expected_alloc_start = allocations * std::mem::size_of::<FooBarBaz>();
        assert_eq!(inner.next, expected_alloc_start);
    }

    #[test]
    fn can_alloc_across_pages() {
        let mut inner = InnerAlloc::new();

        struct Foo {
            _foo: [u8; PAGE_SIZE - 1],
        }

        let layout = Layout::new::<Foo>();
        dbg!(layout);

        assert!(inner.alloc(layout).is_some());

        let expected_limit = inner.page_requests * PAGE_SIZE;
        assert_eq!(inner.upper_limit, expected_limit);

        let expected_alloc_start = 1 * std::mem::size_of::<Foo>();
        assert_eq!(inner.next, expected_alloc_start);

        dbg!(inner);

        // Since this is two bytes it'll push us over to the next page
        let layout = Layout::new::<u16>();
        assert!(inner.alloc(layout).is_some());

        let expected_limit = inner.page_requests * PAGE_SIZE;
        assert_eq!(inner.upper_limit, expected_limit);

        // TODO: Fix size hack
        let expected_alloc_start =
            1 * std::mem::size_of::<Foo>() + 1 * std::mem::size_of::<u16>() + 1;
        assert_eq!(inner.next, expected_alloc_start);
    }

    // TODO: Don't think this actually quite works as expected at the moment...
    #[test]
    fn can_alloc_multiple_pages() {
        let mut inner = InnerAlloc::new();

        struct Foo {
            _foo: [u8; 2 * PAGE_SIZE - 1],
        }

        let layout = Layout::new::<Foo>();
        assert!(inner.alloc(layout).is_some());

        let expected_limit = inner.page_requests * PAGE_SIZE;
        assert_eq!(inner.upper_limit, expected_limit);

        let expected_alloc_start = 1 * std::mem::size_of::<Foo>();
        assert_eq!(inner.next, expected_alloc_start);
    }
}
