// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::storage::{
    self,
    alloc::{
        Allocate,
        AllocateUsing,
        Initialize,
    },
    chunk::SyncChunk,
    Flush,
};

use core::iter::{
    DoubleEndedIterator,
    ExactSizeIterator,
};

/// A contiguous growable array type, written `Vec<T>` but pronounced 'vector'.
///
/// # Note
///
/// Despite the similarity to Rust's `Vec` type this storage `Vec` has many
/// differences in its internal data layout. While it stores its data in contiguous
/// storage slots this does not mean that the data is actually densely stored
/// in memory.
///
/// Also its technical performance characteristics may be different from Rust's
/// `Vec` due to the differences stated above.
///
/// Allows to store up to `2^32` elements and is guaranteed to not reallocate
/// upon pushing new elements to it.
#[derive(Debug)]
pub struct Vec<T> {
    /// The length of the vector.
    len: storage::Value<u32>,
    /// The synchronized cells to operate on the contract storage.
    cells: SyncChunk<T>,
}

/// An iterator over the values of a storage `Vec`.
#[derive(Debug)]
pub struct Iter<'a, T> {
    /// The storage vector to iterate over.
    vec: &'a Vec<T>,
    /// The current begin of the iteration.
    begin: u32,
    /// The current end of the iteration.
    end: u32,
}

impl<'a, T> Iter<'a, T> {
    /// Creates a new iterator for the given storage vector.
    pub(crate) fn new(vec: &'a Vec<T>) -> Self {
        Self {
            vec,
            begin: 0,
            end: vec.len(),
        }
    }
}

impl<T> Flush for Vec<T>
where
    T: parity_codec::Encode + Flush,
{
    fn flush(&mut self) {
        self.len.flush();
        self.cells.flush();
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: parity_codec::Codec,
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

impl<'a, T> ExactSizeIterator for Iter<'a, T> where T: parity_codec::Codec {}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: parity_codec::Codec,
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

impl<T> parity_codec::Encode for Vec<T> {
    fn encode_to<W: parity_codec::Output>(&self, dest: &mut W) {
        self.len.encode_to(dest);
        self.cells.encode_to(dest);
    }
}

impl<T> parity_codec::Decode for Vec<T> {
    fn decode<I: parity_codec::Input>(input: &mut I) -> Option<Self> {
        let len = storage::Value::decode(input)?;
        let cells = SyncChunk::decode(input)?;
        Some(Self { len, cells })
    }
}

impl<T> AllocateUsing for Vec<T> {
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            len: storage::Value::allocate_using(alloc),
            cells: SyncChunk::allocate_using(alloc),
        }
    }
}

impl<T> Initialize for Vec<T> {
    type Args = ();

    fn default_value() -> Option<Self::Args> {
        Some(())
    }

    fn initialize(&mut self, _args: Self::Args) {
        self.len.set(0);
    }
}

impl<T> Vec<T> {
    /// Returns the number of elements in the vector, also referred to as its 'length'.
    pub fn len(&self) -> u32 {
        *self.len.get()
    }

    /// Returns `true` if the vector contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Vec<T>
where
    T: parity_codec::Codec,
{
    /// Returns the given `n` if it is witihn bounds, otherwise `None`.
    fn within_bounds(&self, n: u32) -> Option<u32> {
        if n < self.len() {
            return Some(n)
        }
        None
    }

    /// Returns an iterator over the references of all elements stored in the vector.
    ///
    /// # Note
    ///
    /// - It is **not** recommended to iterate over all elements of a storage vector.
    /// - Try to avoid this if possible or iterate only over a minimal subset of
    ///   all elements using e.g. `Iterator::take(n)`.
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }

    /// Returns a reference to the `n`-th element of the vector.
    ///
    /// Returns `None` if `n` is out of bounds.
    pub fn get(&self, n: u32) -> Option<&T> {
        self.within_bounds(n).and_then(|n| self.cells.get(n))
    }

    /// Returns a mutable reference to the `n`-th element of the vector.
    ///
    /// Returns `None` if `n` is out of bounds.
    pub fn get_mut(&mut self, n: u32) -> Option<&mut T> {
        self.within_bounds(n)
            .and_then(move |n| self.cells.get_mut(n))
    }

    /// Mutates the `n`-th element of the vector.
    ///
    /// Returns a reference to the mutated element.
    /// Returns `None` and won't mutate if `n` out of bounds.
    pub fn mutate<F>(&mut self, n: u32, f: F) -> Option<&T>
    where
        F: FnOnce(&mut T),
    {
        self.within_bounds(n).and_then(move |n| {
            let mut val = self.cells.get_mut(n);
            if let Some(val) = &mut val {
                f(val);
            }
            val.map(|v| &*v)
        })
    }

    /// Appends an element to the back of the vector.
    pub fn push(&mut self, val: T) {
        if self.len() == u32::max_value() {
            panic!(
                "[ink_core::Vec::push] Error: \
                 cannot push more elements than `u32::MAX`"
            )
        }
        let last_index = self.len();
        self.len.set(last_index + 1);
        self.cells.set(last_index, val);
    }

    /// Removes the last element from the vector and returns it,
    /// or `None` if the vector is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None
        }
        let last_index = self.len() - 1;
        self.len.set(last_index);
        self.cells.take(last_index)
    }

    /// Replaces the `n`-th element of the vector and returns its replaced value.
    ///
    /// Returns `None` if `n` is out of bounds.
    pub fn replace<F>(&mut self, n: u32, f: F) -> Option<T>
    where
        F: FnOnce() -> T,
    {
        self.within_bounds(n).and_then(|n| {
            Some(self.cells.put(n, f()).expect(
                "[ink_core::Vec::replace] Error: \
                 expected success due to access within bounds",
            ))
        })
    }

    /// Swaps the `a`-th and the `b`-th elements.
    ///
    /// # Panics
    ///
    /// If one or both indices are out of bounds.
    pub fn swap(&mut self, a: u32, b: u32) {
        // Bail out if both indices are equal.
        if a == b {
            return
        }
        self.within_bounds(a).expect(
            "[ink_core::Vec::swap] Error: \
             expected a to be within bounds",
        );
        self.within_bounds(b).expect(
            "[ink_core::Vec::swap] Error: \
             expected b to be within bounds",
        );
        let item_a = self.cells.take(a).expect(
            "[ink_core::Vec::swap] Error: \
             expected succes due to `a` being within bounds",
        );
        let item_b = self.cells.put(b, item_a).expect(
            "[ink_core::Vec::swap] Error: \
             expected success due to `b` being within bounds",
        );
        self.cells.set(a, item_b);
    }

    /// Removes the `n`-th element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector.
    /// Returns `None` and does not remove if `n` is out of bounds.
    ///
    /// # Note
    ///
    /// This does not preserve ordering, but is O(1).
    pub fn swap_remove(&mut self, n: u32) -> Option<T> {
        if self.is_empty() {
            return None
        }
        self.within_bounds(n)?;
        let popped = self.pop().expect(
            "[ink_core::Vec::swap_remove] Error: \
             expected `Some` value since vector is not empty",
        );
        Some(self.cells.put(n, popped).expect(
            "[ink_core::Vec::swap_remove] Error: \
             expected success since the vector is not empty",
        ))
    }
}

impl<T> core::ops::Index<u32> for Vec<T>
where
    T: parity_codec::Codec,
{
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        self.get(index).expect(
            "[ink_core::Vec::index] Error: \
             expected `index` to be within bounds",
        )
    }
}

impl<T> core::ops::IndexMut<u32> for Vec<T>
where
    T: parity_codec::Codec,
{
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.get_mut(index).expect(
            "[ink_core::Vec::index] Error: \
             expected `index` to be within bounds",
        )
    }
}
