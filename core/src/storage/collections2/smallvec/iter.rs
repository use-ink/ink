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

use super::SmallVec;
use crate::storage::{
    LazyArrayLength,
    PullForward,
    StorageFootprint,
};

/// An iterator over the values of a storage `SmallVec`.
#[derive(Debug)]
pub struct Iter<'a, T, N>
where
    N: LazyArrayLength<T>,
{
    /// The storage vector to iterate over.
    vec: &'a SmallVec<T, N>,
    /// The current begin of the iteration.
    begin: u32,
    /// The current end of the iteration.
    end: u32,
}

impl<'a, T, N> Iter<'a, T, N>
where
    N: LazyArrayLength<T>,
{
    /// Creates a new iterator for the given storage vector.
    pub(crate) fn new(vec: &'a SmallVec<T, N>) -> Self {
        Self {
            vec,
            begin: 0,
            end: vec.len(),
        }
    }
}

impl<'a, T, N> Iterator for Iter<'a, T, N>
where
    T: StorageFootprint + PullForward,
    N: LazyArrayLength<T>,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        if self.begin == self.end {
            return None
        }
        let cur = self.begin;
        self.begin += 1;
        self.vec.get(cur)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.end - self.begin) as usize;
        (remaining, Some(remaining))
    }
}

impl<'a, T, N> ExactSizeIterator for Iter<'a, T, N>
where
    T: StorageFootprint + PullForward,
    N: LazyArrayLength<T>,
{
}

impl<'a, T, N> DoubleEndedIterator for Iter<'a, T, N>
where
    T: StorageFootprint + PullForward,
    N: LazyArrayLength<T>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        if self.begin == self.end {
            return None
        }
        debug_assert_ne!(self.end, 0);
        self.end -= 1;
        self.vec.get(self.end)
    }
}
