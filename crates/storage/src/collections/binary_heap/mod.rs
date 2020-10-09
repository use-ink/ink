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

mod impls;
mod reverse;
mod storage;

#[cfg(test)]
mod tests;

use super::vec::{
    Iter,
    IterMut,
    Vec as StorageVec,
};
use crate::traits::PackedLayout;
pub use reverse::Reverse;

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
    /// The underlying storage vec.
    elems: StorageVec<T>,
}

impl<T> BinaryHeap<T>
where
    T: PackedLayout + Ord,
{
    /// Creates a new empty storage heap.
    pub fn new() -> Self {
        Self {
            elems: StorageVec::new(),
        }
    }

    /// Returns the number of elements in the heap, also referred to as its 'length'.
    pub fn len(&self) -> u32 {
        self.elems.len()
    }

    /// Returns `true` if the heap contains no elements.
    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
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
        self.elems.iter()
    }

    /// Returns an iterator yielding exclusive references to all elements of the heap.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over big heaps.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.elems.iter_mut()
    }

    /// Returns a shared reference to the greatest element of the heap
    ///
    /// Returns `None` if the heap is empty
    pub fn peek(&self) -> Option<&T> {
        self.elems.first()
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
            if right < end && self.elems.get(child) <= self.elems.get(right) {
                child = right;
            }
            // if we are already in order, stop.
            if self.elems.get(pos) >= self.elems.get(child) {
                break
            }
            self.elems.swap(child, pos);
            pos = child;
            child = 2 * pos + 1;
        }
    }

    /// Pops greatest element from the heap and returns it
    ///
    /// Returns `None` if the heap is empty
    pub fn pop(&mut self) -> Option<T> {
        // replace the root of the heap with the last element
        let elem = self.elems.swap_remove(0);
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
        self.elems.clear()
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
            if self.elems.get(pos) <= self.elems.get(parent) {
                break
            }
            self.elems.swap(parent, pos);
            pos = parent;
        }
    }

    /// Pushes the given element to the binary heap.
    pub fn push(&mut self, value: T) {
        let old_len = self.len();
        self.elems.push(value);
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
    /// If `true`, on `drop()` will sift the peeked value down the tree if after mutation it is no
    /// longer the largest value, in order to keep the heap in a consistent state.
    ///
    /// If the peeked value is consumed via `PeekMut::pop()` then this is set to false to prevent
    /// a redundant reorg which would already have happened via `BinaryHeap::pop()`.
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
            .elems
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
            .elems
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
