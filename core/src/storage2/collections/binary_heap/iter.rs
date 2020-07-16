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

use super::BinaryHeap;
use crate::{
    storage2::{
        traits::PackedLayout,
    },
};

/// An iterator over shared references to the elements of a storage vector.
#[derive(Debug, Clone, Copy)]
pub struct Iter<'a, T>
where
    T: PackedLayout + Ord,
{
    /// The storage vector to iterate over.
    heap: &'a BinaryHeap<T>,
    /// The current begin of the iteration.
    begin: u32,
    /// The current end of the iteration.
    end: u32,
}

impl<'a, T> Iter<'a, T>
where
    T: PackedLayout + Ord,
{
    /// Creates a new iterator for the given storage vector.
    pub(crate) fn new(heap: &'a BinaryHeap<T>) -> Self {
        Self {
            heap,
            begin: 0,
            end: heap.len(),
        }
    }

    /// Returns the amount of remaining elements to yield by the iterator.
    fn remaining(&self) -> u32 {
        self.end - self.begin
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: PackedLayout + Ord,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        <Self as Iterator>::nth(self, 0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining() as usize;
        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        self.remaining() as usize
    }

    fn nth(&mut self, _n: usize) -> Option<Self::Item> {
        // todo: implement me!
        None
        // debug_assert!(self.begin <= self.end);
        // let n = n as u32;
        // if self.begin + n >= self.end {
        //     return None
        // }
        // let cur = self.begin + n;
        // self.begin += 1 + n;
        // self.heap.get(cur).expect("access is within bounds").into()
    }
}
