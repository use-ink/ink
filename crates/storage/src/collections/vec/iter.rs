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

use crate::{
    collections::extend_lifetime,
    traits::PackedLayout,
    Vec as StorageVec,
};

/// An iterator over shared references to the elements of a storage vector.
#[derive(Debug, Clone, Copy)]
pub struct Iter<'a, T>
where
    T: PackedLayout,
{
    /// The storage vector to iterate over.
    vec: &'a StorageVec<T>,
    /// The current begin of the iteration.
    begin: u32,
    /// The current end of the iteration.
    end: u32,
}

impl<'a, T> Iter<'a, T>
where
    T: PackedLayout,
{
    /// Creates a new iterator for the given storage vector.
    pub(crate) fn new(vec: &'a StorageVec<T>) -> Self {
        Self {
            vec,
            begin: 0,
            end: vec.len(),
        }
    }

    /// Returns the amount of remaining elements to yield by the iterator.
    fn remaining(&self) -> u32 {
        self.end - self.begin
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: PackedLayout,
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

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        let n = n as u32;
        if self.begin + n >= self.end {
            return None
        }
        let cur = self.begin + n;
        self.begin += 1 + n;
        self.vec.get(cur).expect("access is within bounds").into()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> where T: PackedLayout {}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: PackedLayout,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        <Self as DoubleEndedIterator>::nth_back(self, 0)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        let n = n as u32;
        if self.begin >= self.end.saturating_sub(n) {
            return None
        }
        self.end -= 1 + n;
        self.vec
            .get(self.end)
            .expect("access is within bounds")
            .into()
    }
}

/// An iterator over exclusive references to the elements of a storage vector.
#[derive(Debug)]
pub struct IterMut<'a, T>
where
    T: PackedLayout,
{
    /// The storage vector to iterate over.
    vec: &'a mut StorageVec<T>,
    /// The current begin of the iteration.
    begin: u32,
    /// The current end of the iteration.
    end: u32,
}

impl<'a, T> IterMut<'a, T>
where
    T: PackedLayout,
{
    /// Creates a new iterator for the given storage vector.
    pub(crate) fn new(vec: &'a mut StorageVec<T>) -> Self {
        let len = vec.len();
        Self {
            vec,
            begin: 0,
            end: len,
        }
    }

    /// Returns the amount of remaining elements to yield by the iterator.
    fn remaining(&self) -> u32 {
        self.end - self.begin
    }
}

impl<'a, T> IterMut<'a, T>
where
    T: PackedLayout,
{
    fn get_mut<'b>(&'b mut self, at: u32) -> Option<&'a mut T> {
        self.vec.get_mut(at).map(|value| {
            // SAFETY: We extend the lifetime of the reference here.
            //
            //         This is safe because the iterator yields an exclusive
            //         reference to every element in the iterated vector
            //         just once and also there can be only one such iterator
            //         for the same vector at the same time which is
            //         guaranteed by the constructor of the iterator.
            unsafe { extend_lifetime::<'b, 'a, T>(value) }
        })
    }
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: PackedLayout,
{
    type Item = &'a mut T;

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

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        let n = n as u32;
        if self.begin + n >= self.end {
            return None
        }
        let cur = self.begin + n;
        self.begin += 1 + n;
        self.get_mut(cur).expect("access is within bounds").into()
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> where T: PackedLayout {}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T>
where
    T: PackedLayout,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        <Self as DoubleEndedIterator>::nth_back(self, 0)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        let n = n as u32;
        if self.begin >= self.end.saturating_sub(n) {
            return None
        }
        self.end -= 1 + n;
        self.get_mut(self.end)
            .expect("access is within bounds")
            .into()
    }
}
