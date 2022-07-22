// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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
//! The heap which is used by this allocator is built from pages of Wasm memory (each page is `64KiB`).
//! We will request new pages of memory as needed until we run out of memory, at which point we
//! will crash with an `OOM` error instead of freeing any memory.

use core::alloc::{
    GlobalAlloc,
    Layout,
};

/// A page in Wasm is `64KiB`
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
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // A new page in Wasm is guaranteed to already be zero initialized, so we can just use our
        // regular `alloc` call here and save a bit of work.
        //
        // See: https://webassembly.github.io/spec/core/exec/modules.html#growing-memories
        self.alloc(layout)
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
}

impl InnerAlloc {
    const fn new() -> Self {
        Self {
            next: 0,
            upper_limit: 0,
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(test)] {
            /// Request a `pages` number of page sized sections of Wasm memory. Each page is `64KiB` in size.
            ///
            /// Returns `None` if a page is not available.
            ///
            /// This implementation is only meant to be used for testing, since we cannot (easily)
            /// test the `wasm32` implementation.
            fn request_pages(&mut self, _pages: usize) -> Option<usize> {
                Some(self.upper_limit)
            }
        } else if #[cfg(feature = "std")] {
            fn request_pages(&mut self, _pages: usize) -> Option<usize> {
                unreachable!(
                    "This branch is only used to keep the compiler happy when building tests, and
                     should never actually be called outside of a test run."
                )
            }
        } else if #[cfg(target_arch = "wasm32")] {
            /// Request a `pages` number of pages of Wasm memory. Each page is `64KiB` in size.
            ///
            /// Returns `None` if a page is not available.
            fn request_pages(&mut self, pages: usize) -> Option<usize> {
                let prev_page = core::arch::wasm32::memory_grow(0, pages);
                if prev_page == usize::MAX {
                    return None;
                }

                prev_page.checked_mul(PAGE_SIZE)
            }
        } else {
            compile_error! {
                "ink! only supports compilation as `std` or `no_std` + `wasm32-unknown`"
            }
        }
    }

    /// Tries to allocate enough memory on the heap for the given `Layout`. If there is not enough
    /// room on the heap it'll try and grow it by a page.
    ///
    /// Note: This implementation results in internal fragmentation when allocating across pages.
    fn alloc(&mut self, layout: Layout) -> Option<usize> {
        let alloc_start = self.next;

        let aligned_size = layout.pad_to_align().size();
        let alloc_end = alloc_start.checked_add(aligned_size)?;

        if alloc_end > self.upper_limit {
            let required_pages = required_pages(aligned_size)?;
            let page_start = self.request_pages(required_pages)?;

            self.upper_limit = required_pages
                .checked_mul(PAGE_SIZE)
                .and_then(|pages| page_start.checked_add(pages))?;
            self.next = page_start.checked_add(aligned_size)?;

            Some(page_start)
        } else {
            self.next = alloc_end;
            Some(alloc_start)
        }
    }
}

/// Calculates the number of pages of memory needed for an allocation of `size` bytes.
///
/// This function rounds up to the next page. For example, if we have an allocation of
/// `size = PAGE_SIZE / 2` this function will indicate that one page is required to satisfy
/// the allocation.
#[inline]
fn required_pages(size: usize) -> Option<usize> {
    size.checked_add(PAGE_SIZE - 1)
        .and_then(|num| num.checked_div(PAGE_SIZE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn can_alloc_no_bytes() {
        let mut inner = InnerAlloc::new();

        let layout = Layout::new::<()>();
        assert_eq!(inner.alloc(layout), Some(0));

        let expected_limit =
            PAGE_SIZE * required_pages(layout.pad_to_align().size()).unwrap();
        assert_eq!(inner.upper_limit, expected_limit);

        let expected_alloc_start = size_of::<()>();
        assert_eq!(inner.next, expected_alloc_start);
    }

    #[test]
    fn can_alloc_a_byte() {
        let mut inner = InnerAlloc::new();

        let layout = Layout::new::<u8>();
        assert_eq!(inner.alloc(layout), Some(0));

        let expected_limit =
            PAGE_SIZE * required_pages(layout.pad_to_align().size()).unwrap();
        assert_eq!(inner.upper_limit, expected_limit);

        let expected_alloc_start = size_of::<u8>();
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
        let mut total_size = 0;

        let allocations = 3;
        for _ in 0..allocations {
            assert!(inner.alloc(layout).is_some());
            total_size += layout.pad_to_align().size();
        }

        let expected_limit = PAGE_SIZE * required_pages(total_size).unwrap();
        assert_eq!(inner.upper_limit, expected_limit);

        let expected_alloc_start = allocations * size_of::<FooBarBaz>();
        assert_eq!(inner.next, expected_alloc_start);
    }

    #[test]
    fn can_alloc_across_pages() {
        let mut inner = InnerAlloc::new();

        struct Foo {
            _foo: [u8; PAGE_SIZE - 1],
        }

        // First, let's allocate a struct which is _almost_ a full page
        let layout = Layout::new::<Foo>();
        assert_eq!(inner.alloc(layout), Some(0));

        let expected_limit =
            PAGE_SIZE * required_pages(layout.pad_to_align().size()).unwrap();
        assert_eq!(inner.upper_limit, expected_limit);

        let expected_alloc_start = size_of::<Foo>();
        assert_eq!(inner.next, expected_alloc_start);

        // Now we'll allocate two bytes which will push us over to the next page
        let layout = Layout::new::<u16>();
        assert_eq!(inner.alloc(layout), Some(PAGE_SIZE));

        let expected_limit = 2 * PAGE_SIZE;
        assert_eq!(inner.upper_limit, expected_limit);

        // Notice that we start the allocation on the second page, instead of making use of the
        // remaining byte on the first page
        let expected_alloc_start = PAGE_SIZE + size_of::<u16>();
        assert_eq!(inner.next, expected_alloc_start);
    }

    #[test]
    fn can_alloc_multiple_pages() {
        let mut inner = InnerAlloc::new();

        struct Foo {
            _foo: [u8; 2 * PAGE_SIZE],
        }

        let layout = Layout::new::<Foo>();
        assert_eq!(inner.alloc(layout), Some(0));

        let expected_limit =
            PAGE_SIZE * required_pages(layout.pad_to_align().size()).unwrap();
        assert_eq!(inner.upper_limit, expected_limit);

        let expected_alloc_start = size_of::<Foo>();
        assert_eq!(inner.next, expected_alloc_start);

        // Now we want to make sure that the state of our allocator is correct for any subsequent
        // allocations
        let layout = Layout::new::<u8>();
        assert_eq!(inner.alloc(layout), Some(2 * PAGE_SIZE));

        let expected_limit = 3 * PAGE_SIZE;
        assert_eq!(inner.upper_limit, expected_limit);

        let expected_alloc_start = 2 * PAGE_SIZE + size_of::<u8>();
        assert_eq!(inner.next, expected_alloc_start);
    }
}

#[cfg(test)]
// #[cfg(all(test, feature = "ink-fuzz-tests"))]
mod fuzz_tests {
    use super::*;
    use quickcheck::{
        quickcheck,
        TestResult,
    };
    use std::mem::size_of;

    const FROM_SIZE_ALIGN_EXPECT: &str =
        "The rounded value of `size` cannot be more than `usize::MAX` since we have
        checked that it is a PAGE_SIZE less than `usize::MAX`; Alignment is a
        non-zero, power of two.";

    #[quickcheck]
    fn should_allocate_arbitrary_sized_bytes(n: usize) -> TestResult {
        let mut inner = InnerAlloc::new();

        // If we're going to end up creating an invalid `Layout` we don't want to use these test
        // inputs. We'll check the case where `n` overflows in another test.
        let layout = match Layout::from_size_align(n, size_of::<usize>()) {
            Ok(l) => l,
            Err(_) => return TestResult::discard(),
        };

        let size = layout.pad_to_align().size();
        assert_eq!(
            inner.alloc(layout),
            Some(0),
            "The given pointer for the allocation doesn't match."
        );

        let expected_alloc_start = size;
        assert_eq!(
            inner.next, expected_alloc_start,
            "Our next allocation doesn't match where it should start."
        );

        let expected_limit = PAGE_SIZE * required_pages(size).unwrap();
        assert_eq!(
            inner.upper_limit, expected_limit,
            "The upper bound of our heap doesn't match."
        );

        TestResult::passed()
    }

    #[quickcheck]
    fn should_allocate_regardless_of_alignment_size(
        n: usize,
        align: usize,
    ) -> TestResult {
        let aligns = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512];
        let align = aligns[align % aligns.len()];

        let mut inner = InnerAlloc::new();

        // If we're going to end up creating an invalid `Layout` we don't want to use these test
        // inputs. We'll check the case where `n` overflows in another test.
        let layout = match Layout::from_size_align(n, align) {
            Ok(l) => l,
            Err(_) => return TestResult::discard(),
        };

        let size = layout.pad_to_align().size();
        assert_eq!(
            inner.alloc(layout),
            Some(0),
            "The given pointer for the allocation doesn't match."
        );

        let expected_alloc_start = size;
        assert_eq!(
            inner.next, expected_alloc_start,
            "Our next allocation doesn't match where it should start."
        );

        let expected_limit = PAGE_SIZE * required_pages(size).unwrap();
        assert_eq!(
            inner.upper_limit, expected_limit,
            "The upper bound of our heap doesn't match."
        );

        TestResult::passed()
    }

    /// The idea behind this fuzz test is to check a series of allocation sequences. For
    /// example, we maybe have back to back runs as follows:
    ///
    /// 1. `vec![1, 2, 3]`
    /// 2. `vec![4, 5, 6, 7]`
    /// 3. `vec![8]`
    ///
    /// Each of the vectors represents one sequence of allocations. Within each sequence the
    /// individual size of allocations will be randomly selected by `quickcheck`.
    #[quickcheck]
    fn should_allocate_arbitrary_byte_sequences(sequence: Vec<usize>) -> TestResult {
        let mut inner = InnerAlloc::new();

        if sequence.is_empty() {
            return TestResult::discard()
        }

        // We want to make sure no single allocation is going to overflow, we'll check this
        // case in a different test
        //
        if !sequence
            .iter()
            .all(|n| Layout::from_size_align(*n, size_of::<usize>()).is_ok())
        // .all(|n| n.checked_add(PAGE_SIZE - 1).is_some())
        {
            return TestResult::discard()
        }

        // We can't just use `required_pages(Iterator::sum())` here because it ends up
        // underestimating the pages due to the ceil rounding at each step
        let pages_required = sequence
            .iter()
            .fold(0, |acc, &x| acc + required_pages(x).unwrap());
        let max_pages = required_pages(usize::MAX - PAGE_SIZE + 1).unwrap();

        // We know this is going to end up overflowing, we'll check this case in a different
        // test
        if pages_required > max_pages {
            return TestResult::discard()
        }

        let mut expected_alloc_start = 0;
        let mut total_bytes_requested = 0;
        let mut total_bytes_fragmented = 0;

        for alloc in sequence {
            let layout = Layout::from_size_align(alloc, size_of::<usize>())
                .expect(FROM_SIZE_ALIGN_EXPECT);

            let size = layout.pad_to_align().size();

            let current_page_limit = PAGE_SIZE * required_pages(inner.next).unwrap();
            let is_too_big_for_current_page = inner.next + size > current_page_limit;

            if is_too_big_for_current_page {
                let fragmented_in_current_page = current_page_limit - inner.next;
                total_bytes_fragmented += fragmented_in_current_page;

                // We expect our next allocation to be aligned to the start of the next page
                // boundary
                expected_alloc_start = inner.upper_limit;
            }

            assert_eq!(
                inner.alloc(layout),
                Some(expected_alloc_start),
                "The given pointer for the allocation doesn't match."
            );
            total_bytes_requested += size;

            expected_alloc_start = total_bytes_requested + total_bytes_fragmented;
            assert_eq!(
                inner.next, expected_alloc_start,
                "Our next allocation doesn't match where it should start."
            );

            let pages_required = required_pages(expected_alloc_start).unwrap();
            let expected_limit = PAGE_SIZE * pages_required;
            assert_eq!(
                inner.upper_limit, expected_limit,
                "The upper bound of our heap doesn't match."
            );
        }

        TestResult::passed()
    }

    // For this test we have sequences of allocations which will eventually overflow the maximum
    // amount of pages (in practice this means our heap will be OOM).
    //
    // We don't care about the allocations that succeed (those are checked in other tests), we just
    // care that eventually an allocation doesn't success.
    #[quickcheck]
    fn should_not_allocate_arbitrary_byte_sequences_which_eventually_overflow(
        sequence: Vec<usize>,
    ) -> TestResult {
        let mut inner = InnerAlloc::new();

        if sequence.is_empty() {
            return TestResult::discard()
        }

        // We want to make sure no single allocation is going to overflow, we'll check that
        // case seperately
        if !sequence
            .iter()
            .all(|n| Layout::from_size_align(*n, size_of::<usize>()).is_ok())
        // .all(|n| n.checked_add(PAGE_SIZE - 1).is_some())
        {
            return TestResult::discard()
        }

        // We can't just use `required_pages(Iterator::sum())` here because it ends up
        // underestimating the pages due to the ceil rounding at each step
        let pages_required = sequence
            .iter()
            .fold(0, |acc, &x| acc + required_pages(x).unwrap());
        let max_pages = required_pages(usize::MAX - PAGE_SIZE + 1).unwrap();

        // We want to explicitly test for the case where a series of allocations eventually
        // runs out of pages of memory
        if !(pages_required > max_pages) {
            return TestResult::discard()
        }

        let mut results = vec![];
        for alloc in sequence {
            let layout = Layout::from_size_align(alloc, size_of::<usize>())
                .expect(FROM_SIZE_ALIGN_EXPECT);

            results.push(inner.alloc(layout));
        }

        // Ensure that at least one of the allocations ends up overflowing our calculations.
        assert!(
            results.iter().any(|r| r.is_none()),
            "Expected an allocation to overflow our heap, but this didn't happen."
        );

        TestResult::passed()
    }
}
