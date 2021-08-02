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

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IterMut<'a, T> {
    pub(crate) index: u32,
    pub(crate) range: Range<u32>,
    pub(crate) backing_storage: &'a T,
}

impl<'a, T> IterMut<'a, T> {
    fn current(&self) -> u32 {
        self.range.start + self.index
    }

    fn increment(&mut self) -> Option<u32> {
        let current = self.current();
        if current >= self.range.end {
            None
        } else {
            self.index += 1;
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
        let left = (self.range.end - self.current()) as usize;
        (left, Some(left))
    }

    fn count(self) -> usize {
        (self.range.end - self.current()) as usize
    }

    fn last(self) -> Option<Self::Item> {
        unsafe { self.backing_storage.get_mut(self.range.end) }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.index += n as u32;
        self.next()
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Iter<'a, T> {
    pub(crate) index: u32,
    pub(crate) range: Range<u32>,
    pub(crate) backing_storage: &'a T,
}

impl<'a, T> Iter<'a, T> {
    fn current(&self) -> u32 {
        self.range.start + self.index
    }

    fn increment(&mut self) -> Option<u32> {
        let current = self.current();
        if current >= self.range.end {
            None
        } else {
            self.index += 1;
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
        if let Some(i) = self.increment() {
            self.backing_storage.get(i)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = (self.range.end - self.current()) as usize;
        (left, Some(left))
    }

    fn count(self) -> usize {
        (self.range.end - self.current()) as usize
    }

    fn last(self) -> Option<Self::Item> {
        self.backing_storage.get(self.range.end)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.index += n as u32;
        self.next()
    }
}
