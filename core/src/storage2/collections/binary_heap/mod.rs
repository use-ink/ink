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

//! A priority queue implemented with a binary heap.
//!
//! Insertion and popping the largest element have `O(log(n))` complexity.
//! Checking the largest element is `O(1)`.

mod group;
mod impls;
mod reverse;
mod storage;
mod wrapper;

#[cfg(test)]
mod tests;

use super::vec::Vec as StorageVec;
use crate::storage2::{
    collections::extend_lifetime,
    traits::PackedLayout,
};
pub use reverse::Reverse;

use self::{
    group::Group,
    wrapper::Wrapper,
};

/// A priority queue implemented with a binary heap.
///
/// # Note
///
/// The heap is a *max-heap* by default, i.e. the first element is the largest.
/// Either [`Reverse`] or a custom `Ord` implementation can be used to
/// make `BinaryHeap` a *min-heap*. This makes `heap.pop()` return the smallest
/// value instead of the largest one.
#[derive(Default, PartialEq, Eq, Debug)]
pub struct BinaryHeap<T>
where
    T: PackedLayout + Ord,
{
    /// A collection of groups of elements.
    groups: Wrapper<T>,
}

impl<T> BinaryHeap<T>
where
    T: PackedLayout + Ord,
{
    /// Creates a new empty storage heap.
    pub fn new() -> Self {
        Self {
            groups: Wrapper::new(),
        }
    }

    /// Returns the number of elements in the heap, also referred to as its 'length'.
    pub fn len(&self) -> u32 {
        self.groups.len()
    }

    /// Returns `true` if the heap contains no elements.
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
}

impl<T> BinaryHeap<T>
where
    T: PackedLayout + Ord,
{
    /// Returns an iterator yielding shared references to all elements of the heap.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over large heaps.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter(&self) -> Iter<T> {
        self.groups.iter()
    }

    /// Returns an iterator yielding exclusive references to all elements of the heap.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over big heaps.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.groups.iter_mut()
    }

    /// Returns a shared reference to the greatest element of the heap
    ///
    /// Returns `None` if the heap is empty
    pub fn peek(&self) -> Option<&T> {
        self.groups.first()
    }

    /// Returns an exclusive reference to the greatest element of the heap
    ///
    /// Returns `None` if the heap is empty
    ///
    /// # Note:
    ///
    /// If the `PeekMut` value is leaked, the heap may be in an inconsistent state.
    ///
    /// # Example
    ///
    /// ```
    /// use ink_core::storage2::collections::BinaryHeap;
    /// let mut heap = BinaryHeap::new();
    /// assert!(heap.peek_mut().is_none());
    ///
    /// heap.push(1);
    /// heap.push(5);
    /// heap.push(2);
    /// {
    ///     let mut val = heap.peek_mut().unwrap();
    ///     *val = 0;
    /// }
    /// assert_eq!(heap.peek(), Some(&2));
    /// ```
    pub fn peek_mut(&mut self) -> Option<PeekMut<'_, T>> {
        if self.is_empty() {
            None
        } else {
            Some(PeekMut {
                heap: self,
                sift: true,
            })
        }
    }

    /// Take an element at `pos` and move it down the heap, while its children
    /// are smaller.
    fn sift_down(&mut self, mut pos: u32) {
        let end = self.len();
        let mut child = 2 * pos + 1;
        while child < end {
            let right = child + 1;
            // compare with the greater of the two children
            if right < end && self.groups.get(child) <= self.groups.get(right) {
                child = right;
            }
            // if we are already in order, stop.
            if self.groups.get(pos) >= self.groups.get(child) {
                break
            }
            self.groups.swap(child, pos);
            pos = child;
            child = 2 * pos + 1;
        }
    }

    /// Pops greatest element from the heap and returns it
    ///
    /// Returns `None` if the heap is empty
    pub fn pop(&mut self) -> Option<T> {
        // replace the root of the heap with the last element
        let elem = self.groups.swap_remove(0);
        self.sift_down(0);
        elem
    }

    /// Removes all elements from this heap.
    ///
    /// # Note
    ///
    /// Use this method to clear the vector instead of e.g. iterative `pop()`.
    /// This method performs significantly better and does not actually read
    /// any of the elements (whereas `pop()` does).
    pub fn clear(&mut self) {
        self.groups.clear()
    }
}

impl<T> BinaryHeap<T>
where
    T: PackedLayout + Ord,
{
    /// Take an element at `pos` and move it up the heap, while its parent is
    /// larger.
    fn sift_up(&mut self, mut pos: u32) {
        while pos > 0 {
            let parent = (pos - 1) / 2;
            if self.groups.get(pos) <= self.groups.get(parent) {
                break
            }
            self.groups.swap(parent, pos);
            pos = parent;
        }
    }

    /// Pushes the given element to the binary heap.
    pub fn push(&mut self, value: T) {
        let old_len = self.len();
        self.groups.push(value);
        self.sift_up(old_len)
    }
}

/// Structure wrapping a mutable reference to the greatest item on a
/// [`BinaryHeap`].
///
/// This `struct` is created by the [`BinaryHeap::peek_mut`] method.
pub struct PeekMut<'a, T>
where
    T: 'a + PackedLayout + Ord,
{
    heap: &'a mut BinaryHeap<T>,
    sift: bool,
}

impl<T> Drop for PeekMut<'_, T>
where
    T: PackedLayout + Ord,
{
    fn drop(&mut self) {
        if self.sift {
            self.heap.sift_down(0);
        }
    }
}

impl<T> core::ops::Deref for PeekMut<'_, T>
where
    T: PackedLayout + Ord,
{
    type Target = T;
    fn deref(&self) -> &T {
        self.heap
            .groups
            .first()
            .expect("PeekMut is only instantiated for non-empty heaps")
    }
}

impl<T> core::ops::DerefMut for PeekMut<'_, T>
where
    T: PackedLayout + Ord,
{
    fn deref_mut(&mut self) -> &mut T {
        self.heap
            .groups
            .first_mut()
            .expect("PeekMut is only instantiated for non-empty heaps")
    }
}

impl<'a, T> PeekMut<'a, T>
where
    T: PackedLayout + Ord,
{
    /// Removes the peeked value from the heap and returns it.
    pub fn pop(mut this: PeekMut<'a, T>) -> T {
        let value = this
            .heap
            .pop()
            .expect("PeekMut is only instantiated for non-empty heaps");
        this.sift = false;
        value
    }
}

/// An iterator over shared references to the elements of a storage vector.
#[derive(Debug, Clone, Copy)]
pub struct Iter<'a, T>
where
    T: PackedLayout + Ord,
{
    /// The storage vector to iterate over.
    elems: &'a StorageVec<Group<T>>,
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
    pub(crate) fn new(elems: &'a StorageVec<Group<T>>) -> Self {
        Self {
            elems,
            begin: 0,
            end: elems.len(),
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

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        let n = n as u32;
        if self.begin + n >= self.end {
            return None
        }
        let cur = self.begin + n;
        self.begin += 1 + n;

        let group_index = group::get_group_index(cur);
        self.elems
            .get(group_index)
            .expect("access is out of bounds")
            .as_ref(group_index)
    }
}

/// An iterator over exclusive references to the elements of a storage vector.
#[derive(Debug)]
pub struct IterMut<'a, T>
where
    T: PackedLayout + Ord,
{
    /// The storage vector to iterate over.
    elems: &'a mut StorageVec<Group<T>>,
    /// The current begin of the iteration.
    begin: u32,
    /// The current end of the iteration.
    end: u32,
}

impl<'a, T> IterMut<'a, T>
where
    T: PackedLayout + Ord,
{
    /// Creates a new iterator for the given storage vector.
    pub(crate) fn new(elems: &'a mut StorageVec<Group<T>>) -> Self {
        let end = elems.len();
        Self {
            elems,
            begin: 0,
            end,
        }
    }

    /// Returns the amount of remaining elements to yield by the iterator.
    fn remaining(&self) -> u32 {
        self.end - self.begin
    }
}

impl<'a, T> IterMut<'a, T>
where
    T: PackedLayout + Ord,
{
    fn get_mut<'b>(&'b mut self, at: u32) -> Option<&'a mut T> {
        let group_index = group::get_group_index(at);
        let group = self
            .elems
            .get_mut(group_index)
            .expect("access is within bounds");
        group.get_mut(at).as_mut().map(|value| {
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
    T: PackedLayout + Ord,
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
