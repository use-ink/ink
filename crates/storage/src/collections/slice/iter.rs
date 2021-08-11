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

use crate::collections::slice::ContiguousStorage;
use core::ops::Range;

/// Iterator for the [`SliceMut::iter_mut`](crate::collections::slice::SliceMut) method.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IterMut<'a, T> {
    /// The bounds of the iterator. Since `IterMut` is a double ended iterator, either `bounds.start`
    /// or `bounds.end` may be incremented/decremented.
    bounds: Range<u32>,
    /// The underlying storage structure, such as `LazyIndexMap` or `LazyArray`.
    backing_storage: &'a T,
}

impl<'a, T> IterMut<'a, T> {
    /// Returns an iterator over the `backing_storage` from the start of the bounds (inclusive) to
    /// the end (exclusive).
    ///
    /// # Safety
    /// The caller must ensure that no overlapping Slices or Iterators exist for the given range, as
    /// `IterMut` provides mutable references to the items within the bounds.
    pub unsafe fn new(bounds: Range<u32>, backing_storage: &'a T) -> Self
    where
        T: ContiguousStorage,
    {
        IterMut {
            bounds,
            backing_storage,
        }
    }

    fn next_index(&mut self) -> Option<u32> {
        let current = self.bounds.start;
        if current >= self.bounds.end {
            None
        } else {
            self.bounds.start += 1;
            Some(current)
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: ContiguousStorage,
{
    type Item = &'a mut <T as ContiguousStorage>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.next_index()?;

        // Safety: the iterator has exclusive access to the range of cells through the mutable receiver,
        // and the contract of `IterMut::new`.
        unsafe { self.backing_storage.get_mut(i) }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.len();
        (left, Some(left))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(self) -> Option<Self::Item> {
        if self.bounds.start >= self.bounds.end {
            return None
        }
        // Safety: The iterator has exclusive access to the range in the underlying backing_storage,
        // and the above bounds check ensures that we aren't fetching an out-of-bounds but valid item,
        // even when iterating from the back.
        unsafe { self.backing_storage.get_mut(self.bounds.end - 1) }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.bounds.start += n as u32;
        self.next()
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T>
where
    T: ContiguousStorage,
{
    fn len(&self) -> usize {
        (self.bounds.end - self.bounds.start) as usize
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T>
where
    T: ContiguousStorage,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.bounds.start >= self.bounds.end {
            return None
        }
        let item =
            // Safety: we have exclusive access to the `backing_storage` through the mutable receiver,
            // and the contract of `SliceMut`.
            unsafe { self.backing_storage.get_mut(self.bounds.end - 1) };
        self.bounds.end -= 1;
        item
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.bounds.end = self.bounds.end.saturating_sub(n as u32);
        self.next_back()
    }
}

/// Iterator for the [`Slice::iter`](crate::collections::slice::Slice) method.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Iter<'a, T> {
    /// The bounds of the iterator. Since `Iter` is a double ended iterator, either `bounds.start`
    /// or `bounds.end` may be incremented/decremented.
    bounds: Range<u32>,
    /// The underlying storage structure, such as `LazyIndexMap` or `LazyArray`.
    backing_storage: &'a T,
}

impl<'a, T> Iter<'a, T> {
    /// Returns an iterator over the `backing_storage` from the start of the bounds (inclusive) to
    /// the end (exclusive).
    pub fn new(bounds: Range<u32>, backing_storage: &'a T) -> Self
    where
        T: ContiguousStorage,
    {
        Iter {
            bounds,
            backing_storage,
        }
    }

    fn next_index(&mut self) -> Option<u32> {
        let current = self.bounds.start;
        if current >= self.bounds.end {
            None
        } else {
            self.bounds.start += 1;
            Some(current)
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: ContiguousStorage,
{
    type Item = &'a <T as ContiguousStorage>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.next_index()?;
        self.backing_storage.get(i)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.len();
        (left, Some(left))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(self) -> Option<Self::Item> {
        if self.bounds.start >= self.bounds.end {
            return None
        }
        self.backing_storage.get(self.bounds.end - 1)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.bounds.start += n as u32;
        self.next()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T>
where
    T: ContiguousStorage,
{
    fn len(&self) -> usize {
        (self.bounds.end - self.bounds.start) as usize
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: ContiguousStorage,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.bounds.start >= self.bounds.end {
            return None
        }
        let item = self.backing_storage.get(self.bounds.end - 1);
        self.bounds.end -= 1;
        item
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.bounds.end = self.bounds.end.saturating_sub(n as u32);
        self.next_back()
    }
}
