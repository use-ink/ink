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

use super::{
    Entry,
    Stash,
};
use crate::{
    collections::extend_lifetime,
    traits::PackedLayout,
};

/// An iterator over shared references to the elements of a storage stash.
#[derive(Debug, Clone, Copy)]
pub struct Iter<'a, T>
where
    T: PackedLayout,
{
    /// The storage stash to iterate over.
    stash: &'a Stash<T>,
    /// The number of already yielded elements.
    ///
    /// # Note
    ///
    /// This is important to make this iterator an `ExactSizeIterator`.
    yielded: u32,
    /// The current begin of the iteration.
    begin: u32,
    /// The current end of the iteration.
    end: u32,
}

impl<'a, T> Iter<'a, T>
where
    T: PackedLayout,
{
    /// Creates a new iterator for the given storage stash.
    pub(crate) fn new(stash: &'a Stash<T>) -> Self {
        Self {
            stash,
            yielded: 0,
            begin: 0,
            end: stash.len_entries(),
        }
    }

    /// Returns the amount of remaining elements to yield by the iterator.
    fn remaining(&self) -> u32 {
        self.stash.len() - self.yielded
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: PackedLayout,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            debug_assert!(self.begin <= self.end);
            if self.begin == self.end {
                return None
            }
            let cur = self.begin;
            self.begin += 1;
            match self.stash.get(cur) {
                Some(value) => {
                    self.yielded += 1;
                    return Some(value)
                }
                None => continue,
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining() as usize;
        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        self.remaining() as usize
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> where T: PackedLayout {}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: PackedLayout,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            debug_assert!(self.begin <= self.end);
            if self.begin == self.end {
                return None
            }
            debug_assert_ne!(self.end, 0);
            self.end -= 1;
            match self.stash.get(self.end) {
                Some(value) => {
                    self.yielded += 1;
                    return Some(value)
                }
                None => continue,
            }
        }
    }
}

/// An iterator over exclusive references to the elements of a storage stash.
#[derive(Debug)]
pub struct IterMut<'a, T>
where
    T: PackedLayout,
{
    /// The storage stash to iterate over.
    stash: &'a mut Stash<T>,
    /// The number of already yielded elements.
    ///
    /// # Note
    ///
    /// This is important to make this iterator an `ExactSizeIterator`.
    yielded: u32,
    /// The current begin of the iteration.
    begin: u32,
    /// The current end of the iteration.
    end: u32,
}

impl<'a, T> IterMut<'a, T>
where
    T: PackedLayout,
{
    /// Creates a new iterator for the given storage stash.
    pub(crate) fn new(stash: &'a mut Stash<T>) -> Self {
        let len = stash.len_entries();
        Self {
            stash,
            yielded: 0,
            begin: 0,
            end: len,
        }
    }

    /// Returns the amount of remaining elements to yield by the iterator.
    fn remaining(&self) -> u32 {
        self.stash.len() - self.yielded
    }
}

impl<'a, T> IterMut<'a, T>
where
    T: PackedLayout,
{
    fn get_mut<'b>(&'b mut self, at: u32) -> Option<&'a mut T> {
        self.stash.get_mut(at).map(|value| {
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
        loop {
            debug_assert!(self.begin <= self.end);
            if self.begin == self.end {
                return None
            }
            let cur = self.begin;
            self.begin += 1;
            match self.get_mut(cur) {
                Some(value) => {
                    self.yielded += 1;
                    return Some(value)
                }
                None => continue,
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining() as usize;
        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        self.remaining() as usize
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> where T: PackedLayout {}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T>
where
    T: PackedLayout,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            debug_assert!(self.begin <= self.end);
            if self.begin == self.end {
                return None
            }
            debug_assert_ne!(self.end, 0);
            self.end -= 1;
            match self.get_mut(self.end) {
                Some(value) => {
                    self.yielded += 1;
                    return Some(value)
                }
                None => continue,
            }
        }
    }
}

/// An iterator over shared references to the entries of a storage stash.
///
/// # Note
///
/// This is an internal API and mainly used for testing the storage stash.
#[derive(Debug, Clone, Copy)]
pub struct Entries<'a, T>
where
    T: PackedLayout,
{
    /// The storage stash to iterate over.
    stash: &'a Stash<T>,
    /// The current begin of the iteration.
    begin: u32,
    /// The current end of the iteration.
    end: u32,
}

impl<'a, T> Entries<'a, T>
where
    T: PackedLayout,
{
    /// Creates a new iterator for the given storage stash.
    pub(crate) fn new(stash: &'a Stash<T>) -> Self {
        let len = stash.len_entries();
        Self {
            stash,
            begin: 0,
            end: len,
        }
    }
}

impl<'a, T> Iterator for Entries<'a, T>
where
    T: PackedLayout,
{
    type Item = &'a Entry<T>;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        if self.begin == self.end {
            return None
        }
        let cur = self.begin;
        self.begin += 1;
        let entry = self
            .stash
            .entries
            .get(cur)
            .expect("iterator indices are within bounds");
        Some(entry)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.end - self.begin) as usize;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for Entries<'a, T> where T: PackedLayout {}

impl<'a, T> DoubleEndedIterator for Entries<'a, T>
where
    T: PackedLayout,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        if self.begin == self.end {
            return None
        }
        debug_assert_ne!(self.end, 0);
        self.end -= 1;
        let entry = self
            .stash
            .entries
            .get(self.end)
            .expect("iterator indices are within bounds");
        Some(entry)
    }
}
