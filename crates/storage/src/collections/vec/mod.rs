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

//! A storage vector used to store elements in a contiguous sequenced order.
//!
//! This is by default the go-to collection for most smart contracts if there
//! are not special requirements to the storage data structure.

mod impls;
mod iter;
mod storage;

#[cfg(test)]
mod tests;

pub use self::iter::{
    Iter,
    IterMut,
};
use crate::{
    lazy::{
        Lazy,
        LazyIndexMap,
    },
    traits::PackedLayout,
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
pub struct Vec<T>
where
    T: PackedLayout,
{
    /// The length of the vector.
    len: Lazy<u32>,
    /// The synchronized cells to operate on the contract storage.
    elems: LazyIndexMap<T>,
}

/// The index is out of the bounds of this vector.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexOutOfBounds;

impl<T> Default for Vec<T>
where
    T: PackedLayout,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Vec<T>
where
    T: PackedLayout,
{
    /// Creates a new empty storage vector.
    pub fn new() -> Self {
        Self {
            len: Lazy::new(0),
            elems: LazyIndexMap::new(),
        }
    }

    /// Returns the number of elements in the vector, also referred to as its 'length'.
    pub fn len(&self) -> u32 {
        *self.len
    }

    /// Returns `true` if the vector contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Vec<T>
where
    T: PackedLayout,
{
    /// Clears the underlying storage cells of the storage vector.
    ///
    /// # Note
    ///
    /// This completely invalidates the storage vector's invariants about
    /// the contents of its associated storage region.
    ///
    /// This API is used for the `Drop` implementation of [`Vec`] as well as
    /// for the [`SpreadLayout::clear_spread`] trait implementation.
    fn clear_cells(&self) {
        if self.elems.key().is_none() {
            // We won't clear any storage if we are in lazy state since there
            // probably has not been any state written to storage, yet.
            return
        }
        for index in 0..self.len() {
            self.elems.clear_packed_at(index);
        }
    }
}

impl<T> Vec<T>
where
    T: PackedLayout,
{
    /// Returns an iterator yielding shared references to all elements of the vector.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over big storage vectors.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }

    /// Returns an iterator yielding exclusive references to all elements of the vector.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over big storage vectors.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut::new(self)
    }

    /// Returns the index if it is within bounds or `None` otherwise.
    fn within_bounds(&self, index: u32) -> Option<u32> {
        if index < self.len() {
            return Some(index)
        }
        None
    }

    /// Returns a shared reference to the first element if any.
    pub fn first(&self) -> Option<&T> {
        if self.is_empty() {
            return None
        }
        self.get(0)
    }

    /// Returns a shared reference to the last element if any.
    pub fn last(&self) -> Option<&T> {
        if self.is_empty() {
            return None
        }
        let last_index = self.len() - 1;
        self.get(last_index)
    }

    /// Returns a shared reference to the indexed element.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn get(&self, index: u32) -> Option<&T> {
        self.within_bounds(index)
            .and_then(|index| self.elems.get(index))
    }
}

impl<T> Vec<T>
where
    T: PackedLayout,
{
    /// Appends an element to the back of the vector.
    pub fn push(&mut self, value: T) {
        assert!(
            self.len() < core::u32::MAX,
            "cannot push more elements into the storage vector"
        );
        let last_index = self.len();
        *self.len += 1;
        self.elems.put(last_index, Some(value));
    }
}

impl<T> Vec<T>
where
    T: PackedLayout,
{
    /// Pops the last element from the vector and returns it.
    //
    /// Returns `None` if the vector is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None
        }
        let last_index = self.len() - 1;
        *self.len = last_index;
        self.elems.put_get(last_index, None)
    }

    /// Pops the last element from the vector and immediately drops it.
    ///
    /// Returns `Some(())` if an element has been removed and `None` otherwise.
    ///
    /// # Note
    ///
    /// This operation is a bit more efficient than [`Vec::pop`]
    /// since it avoids reading from contract storage in some use cases.
    pub fn pop_drop(&mut self) -> Option<()> {
        if self.is_empty() {
            return None
        }
        let last_index = self.len() - 1;
        *self.len = last_index;
        self.elems.put(last_index, None);
        Some(())
    }

    /// Returns an exclusive reference to the first element if any.
    pub fn first_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None
        }
        self.get_mut(0)
    }

    /// Returns an exclusive reference to the last element if any.
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None
        }
        let last_index = self.len() - 1;
        self.get_mut(last_index)
    }

    /// Returns an exclusive reference to the indexed element.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn get_mut(&mut self, index: u32) -> Option<&mut T> {
        self.within_bounds(index)
            .and_then(move |index| self.elems.get_mut(index))
    }

    /// Swaps the elements at the given indices.
    ///
    /// # Panics
    ///
    /// If one or both indices are out of bounds.
    pub fn swap(&mut self, a: u32, b: u32) {
        assert!(
            a < self.len() && b < self.len(),
            "indices are out of bounds"
        );
        self.elems.swap(a, b)
    }

    /// Removes the indexed element from the vector and returns it.
    ///
    /// The last element of the vector is put into the indexed slot.
    /// Returns `None` and does not mutate the vector if the index is out of bounds.
    ///
    /// # Note
    ///
    /// This operation does not preserve ordering but is constant time.
    pub fn swap_remove(&mut self, n: u32) -> Option<T> {
        if self.is_empty() {
            return None
        }
        self.elems.swap(n, self.len() - 1);
        self.pop()
    }

    /// Removes the indexed element from the vector.
    ///
    /// The last element of the vector is put into the indexed slot.
    /// Returns `Some(())` if an element has been removed and `None` otherwise.
    ///
    /// # Note
    ///
    /// This operation should be preferred over [`Vec::swap_remove`] if there is
    /// no need to return the removed element since it avoids a contract storage
    /// read for some use cases.
    pub fn swap_remove_drop(&mut self, n: u32) -> Option<()> {
        if self.is_empty() {
            return None
        }
        self.elems.put(n, None);
        let last_index = self.len() - 1;
        let last = self.elems.put_get(last_index, None);
        self.elems.put(n, last);
        *self.len = last_index;
        Some(())
    }

    /// Sets the elements at the given index to the new value.
    ///
    /// Won't return the old element back to the caller.
    /// Prefer this operation over other method of overriding an element
    /// in the storage vector since this is more efficient.
    #[inline]
    pub fn set(&mut self, index: u32, new_value: T) -> Result<(), IndexOutOfBounds> {
        if self.within_bounds(index).is_none() {
            return Err(IndexOutOfBounds)
        }
        self.elems.put(index, Some(new_value));
        Ok(())
    }

    /// Removes all elements from this vector.
    ///
    /// # Note
    ///
    /// Use this method to clear the vector instead of e.g. iterative `pop()`.
    /// This method performs significantly better and does not actually read
    /// any of the elements (whereas `pop()` does).
    pub fn clear(&mut self) {
        if self.is_empty() {
            return
        }
        for index in 0..self.len() {
            self.elems.put(index, None);
        }
        *self.len = 0;
    }
}
