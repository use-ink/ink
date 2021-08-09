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
    pub(crate) range: Range<u32>,
    pub(crate) backing_storage: &'a T,
}

impl<'a, T> IterMut<'a, T> {
    fn current(&self) -> u32 {
        self.range.start
    }

    fn increment(&mut self) -> Option<u32> {
        let current = self.current();
        if current >= self.range.end {
            None
        } else {
            self.range.start += 1;
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
        if let Some(i) = self.increment() {
            unsafe { self.backing_storage.get_mut(i) }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.len();
        (left, Some(left))
    }

    fn count(self) -> usize {
        (self.range.end - self.current()) as usize
    }

    fn last(self) -> Option<Self::Item> {
        unsafe { self.backing_storage.get_mut(self.range.end - 1) }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.range.start += n as u32;
        self.next()
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T>
where
    T: ContiguousStorage,
{
    fn len(&self) -> usize {
        (self.range.end - self.current()) as usize
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T>
where
    T: ContiguousStorage,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.range.start >= self.range.end {
            return None
        }
        let item =
            // Safety: we have exclusive access to the `backing_storage` through the mutable receiver,
            // and the contract of `SliceMut`.
            unsafe { self.backing_storage.get_mut(self.range.end - 1) };
        self.range.end -= 1;
        item
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.range.end = self.range.end.saturating_sub(n as u32);
        self.next_back()
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Iter<'a, T> {
    pub(crate) range: Range<u32>,
    pub(crate) backing_storage: &'a T,
}

impl<'a, T> Iter<'a, T> {
    fn current(&self) -> u32 {
        self.range.start
    }

    fn increment(&mut self) -> Option<u32> {
        let current = self.current();
        if current >= self.range.end {
            None
        } else {
            self.range.start += 1;
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
        let i = self.increment()?;
        self.backing_storage.get(i)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = (self.range.end - self.current()) as usize;
        (left, Some(left))
    }

    fn count(self) -> usize {
        (self.range.end - self.current()) as usize
    }

    fn last(self) -> Option<Self::Item> {
        self.backing_storage.get(self.range.end - 1)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.range.start += n as u32;
        self.next()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T>
where
    T: ContiguousStorage,
{
    fn len(&self) -> usize {
        (self.range.end - self.current()) as usize
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: ContiguousStorage,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.range.start >= self.range.end {
            return None
        }
        let item = self.backing_storage.get(self.range.end - 1);
        self.range.end -= 1;
        item
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.range.end = self.range.end.saturating_sub(n as u32);
        self.next_back()
    }
}
