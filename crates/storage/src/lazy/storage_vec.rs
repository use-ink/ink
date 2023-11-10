// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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
//! are no special requirements to the storage data structure.

//mod impls;
//mod iter;
//mod storage;
//
//#[cfg(test)]
//mod tests;
//
//#[cfg(all(test, feature = "ink-fuzz-tests"))]
//mod fuzz_tests;

use core::iter::{Extend, FromIterator};
use ink_storage_traits::{AutoKey, Packed, StorageKey};

//pub use self::iter::{Iter, IterMut};
use crate::{extend_lifetime, lazy::LazyIndexMap};

/// A contiguous growable array type, written `Vec<T>` but pronounced "vector".
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
pub struct Vec<T, KeyType: StorageKey = AutoKey>
where
    T: Packed,
{
    /// The length of the vector.
    len: Option<u32>,
    /// The synchronized cells to operate on the contract storage.
    elems: LazyIndexMap<T, KeyType>,
}

/// The index is out of the bounds of this vector.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexOutOfBounds;

impl<T, KeyType> Default for Vec<T, KeyType>
where
    KeyType: StorageKey,
    T: Packed,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, KeyType> Vec<T, KeyType>
where
    KeyType: StorageKey,
    T: Packed,
{
    /// Creates a new empty storage vector.
    pub fn new() -> Self {
        Self {
            len: None,
            elems: LazyIndexMap::new(),
        }
    }

    /// Returns the number of elements in the vector, also referred to as its length.
    pub fn len(&self) -> u32 {
        self.len.unwrap_or_else(|| {
            ink_env::get_contract_storage(&KeyType::KEY)
                .expect("u32 must always fit into the buffer")
                .unwrap_or(u32::MIN)
        })
    }

    /// Returns `true` if the vector contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T, KeyType> Vec<T, KeyType>
where
    KeyType: StorageKey,
    T: Packed,
{
    /// Clears the underlying storage cells of the storage vector.
    ///
    /// # Note
    ///
    /// This completely invalidates the storage vector's invariants about
    /// the contents of its associated storage region.
    ///
    /// This API is used for the `Drop` implementation of [`Vec`] as well as
    /// for the [`SpreadLayout::clear_spread`][`crate::traits::SpreadLayout::clear_spread`]
    /// trait implementation.
    fn clear_cells(&self) {
        let len = self.len();
        let _ = ink_env::clear_contract_storage(&KeyType::KEY);

        for index in 0..len {
            self.elems.clear_packed_at(index);
        }
    }
}

impl<T, KeyType> Vec<T, KeyType>
where
    KeyType: StorageKey,
    T: Packed,
{
    /// Returns an iterator yielding shared references to all elements of the vector.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over big storage vectors.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter(&self) -> Iter<T, KeyType> {
        Iter::new(self)
    }

    /// Returns an iterator yielding exclusive references to all elements of the vector.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over big storage vectors.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter_mut(&mut self) -> IterMut<T, KeyType> {
        IterMut::new(self)
    }

    /// Returns the index if it is within bounds or `None` otherwise.
    fn within_bounds(&self, index: u32) -> Option<u32> {
        if index < self.len() {
            return Some(index);
        }
        None
    }

    /// Returns a shared reference to the first element if any.
    pub fn first(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        self.get(0)
    }

    /// Returns a shared reference to the last element if any.
    pub fn last(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
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

impl<T, KeyType> Vec<T, KeyType>
where
    KeyType: StorageKey,
    T: Packed,
{
    /// Appends an element to the back of the vector.
    pub fn push(&mut self, value: T) {
        let last_index = self.len();
        assert!(
            last_index < core::u32::MAX,
            "cannot push more elements into the storage vector"
        );
        self.len = Some(last_index.checked_add(1).unwrap());
        self.elems.put(last_index, Some(value));
    }

    /// Binary searches this sorted vector for a given element.
    ///
    /// If the value is found then [`Result::Ok`] is returned, containing the
    /// index of the matching element. If there are multiple matches, then any
    /// one of the matches could be returned. If the value is not found then
    /// [`Result::Err`] is returned, containing the index where a matching
    /// element could be inserted while maintaining sorted order.
    ///
    /// See also [`binary_search_by`], [`binary_search_by_key`].
    ///
    /// [`binary_search_by`]: Vec::binary_search_by
    /// [`binary_search_by_key`]: Vec::binary_search_by_key
    ///
    /// # Examples
    ///
    /// Looks up a series of four elements. The first is found, with a
    /// uniquely determined position; the second and third are not
    /// found; the fourth could match any position in `[1, 4]`.
    ///
    /// ```ignore
    /// # // Tracking issue [#1119]: We currently ignore this test since we stopped exposing
    /// # // `StorageVec` publicly.
    /// use ink_storage::Vec as StorageVec;
    ///
    /// let s: StorageVec<i32> = [0, 1, 1, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55]
    ///     .into_iter()
    ///     .collect();
    ///
    /// assert_eq!(s.binary_search(&13),  Ok(9));
    /// assert_eq!(s.binary_search(&4),   Err(7));
    /// assert_eq!(s.binary_search(&100), Err(13));
    /// let r = s.binary_search(&1);
    /// assert!(match r { Ok(1..=4) => true, _ => false, });
    /// ```
    #[inline]
    pub fn binary_search(&self, x: &T) -> Result<u32, u32>
    where
        T: Ord,
    {
        self.binary_search_by(|p| p.cmp(x))
    }

    /// Binary searches this sorted vector with a comparator function.
    ///
    /// The comparator function should implement an order consistent
    /// with the sort order of the underlying vector, returning an
    /// order code that indicates whether its argument is `Less`,
    /// `Equal` or `Greater` the desired target.
    ///
    /// If the value is found then [`Result::Ok`] is returned, containing the
    /// index of the matching element. If there are multiple matches, then any
    /// one of the matches could be returned. If the value is not found then
    /// [`Result::Err`] is returned, containing the index where a matching
    /// element could be inserted while maintaining sorted order.
    ///
    /// See also [`binary_search`], [`binary_search_by_key`].
    ///
    /// [`binary_search`]: Vec::binary_search
    /// [`binary_search_by_key`]: Vec::binary_search_by_key
    ///
    /// # Examples
    ///
    /// Looks up a series of four elements. The first is found, with a
    /// uniquely determined position; the second and third are not
    /// found; the fourth could match any position in `[1, 4]`.
    ///
    /// ```ignore
    /// # // Tracking issue [#1119]: We currently ignore this test since we stopped exposing
    /// # // `StorageVec` publicly.
    /// use ink_storage::Vec as StorageVec;
    ///
    /// let s: StorageVec<i32> = [0, 1, 1, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55]
    ///     .into_iter()
    ///     .collect();
    ///
    /// let seek = 13;
    /// assert_eq!(s.binary_search_by(|probe| probe.cmp(&seek)), Ok(9));
    /// let seek = 4;
    /// assert_eq!(s.binary_search_by(|probe| probe.cmp(&seek)), Err(7));
    /// let seek = 100;
    /// assert_eq!(s.binary_search_by(|probe| probe.cmp(&seek)), Err(13));
    /// let seek = 1;
    /// let r = s.binary_search_by(|probe| probe.cmp(&seek));
    /// assert!(match r { Ok(1..=4) => true, _ => false, });
    /// ```
    // The binary_search implementation is ported from
    // https://github.com/rust-lang/rust/blob/c5e344f7747dbd7e7d4b209e3c480deb5979a56f/library/core/src/slice/mod.rs#L2191
    // and attempts to remain as close to the source as possible.
    #[inline]
    pub fn binary_search_by<'a, F>(&'a self, mut f: F) -> Result<u32, u32>
    where
        F: FnMut(&'a T) -> core::cmp::Ordering,
    {
        use core::cmp::Ordering::*;

        let mut size = self.len();
        let mut left = 0;
        let mut right = size;
        while left < right {
            let mid = left + size / 2;

            // the call is made safe by the following invariants:
            // - `mid >= 0`
            // - `mid < size`: `mid` is limited by `[left; right)` bound.
            let cmp = f(&self[mid]);

            // The reason why we use if/else control flow rather than match
            // is because match reorders comparison operations, which is perf sensitive.
            if cmp == Less {
                left = mid + 1;
            } else if cmp == Greater {
                right = mid;
            } else {
                return Ok(mid);
            }

            size = right - left;
        }
        Err(left)
    }

    /// Binary searches this sorted vector with a key extraction function.
    ///
    /// If the value is found then [`Result::Ok`] is returned, containing the
    /// index of the matching element. If there are multiple matches, then any
    /// one of the matches could be returned. If the value is not found then
    /// [`Result::Err`] is returned, containing the index where a matching
    /// element could be inserted while maintaining sorted order.
    ///
    /// See also [`binary_search`], [`binary_search_by`].
    ///
    /// [`binary_search`]: Vec::binary_search
    /// [`binary_search_by`]: Vec::binary_search_by
    ///
    /// # Examples
    ///
    /// Looks up a series of four elements in a vector of pairs sorted by
    /// their second elements. The first is found, with a uniquely
    /// determined position; the second and third are not found; the
    /// fourth could match any position in `[1, 4]`.
    ///
    /// ```ignore
    /// # // Tracking issue [#1119]: We currently ignore this test since we stopped exposing
    /// # // `StorageVec` publicly.
    /// use ink_storage::Vec as StorageVec;
    ///
    /// let s: StorageVec<(i32, i32)> = [
    ///     (0, 0),
    ///     (2, 1),
    ///     (4, 1),
    ///     (5, 1),
    ///     (3, 1),
    ///     (1, 2),
    ///     (2, 3),
    ///     (4, 5),
    ///     (5, 8),
    ///     (3, 13),
    ///     (1, 21),
    ///     (2, 34),
    ///     (4, 55),
    /// ]
    /// .into_iter()
    /// .collect();
    ///
    /// assert_eq!(s.binary_search_by_key(&13, |&(a, b)| b),  Ok(9));
    /// assert_eq!(s.binary_search_by_key(&4, |&(a, b)| b),   Err(7));
    /// assert_eq!(s.binary_search_by_key(&100, |&(a, b)| b), Err(13));
    /// let r = s.binary_search_by_key(&1, |&(a, b)| b);
    /// assert!(match r { Ok(1..=4) => true, _ => false, });
    /// ```
    #[inline]
    pub fn binary_search_by_key<'a, B, F>(&'a self, b: &B, mut f: F) -> Result<u32, u32>
    where
        F: FnMut(&'a T) -> B,
        B: Ord,
    {
        self.binary_search_by(|k| f(k).cmp(b))
    }
}

impl<T, KeyType> Vec<T, KeyType>
where
    KeyType: StorageKey,
    T: Packed,
{
    /// Pops the last element from the vector and returns it.
    //
    /// Returns `None` if the vector is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let last_index = self.len() - 1;
        self.len = Some(last_index);
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
            return None;
        }
        let last_index = self.len() - 1;
        self.len = Some(last_index);
        self.elems.put(last_index, None);
        Some(())
    }

    /// Returns an exclusive reference to the first element if any.
    pub fn first_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None;
        }
        self.get_mut(0)
    }

    /// Returns an exclusive reference to the last element if any.
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None;
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
            return None;
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
            return None;
        }
        self.elems.put(n, None);
        let last_index = self.len() - 1;
        let last = self.elems.put_get(last_index, None);
        self.elems.put(n, last);
        self.len = Some(last_index);
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
            return Err(IndexOutOfBounds);
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
            return;
        }
        for index in 0..self.len() {
            self.elems.put(index, None);
        }
        self.len = Some(0);
    }

    pub fn write(&self) {
        if self.is_empty() {
            return;
        }

        ink_env::set_contract_storage(&KeyType::KEY, &self.len());
        self.elems.write();
    }
}

impl<T, KeyType> Drop for Vec<T, KeyType>
where
    KeyType: StorageKey,
    T: Packed,
{
    fn drop(&mut self) {
        self.clear_cells();
    }
}

impl<T, KeyType> core::ops::Index<u32> for Vec<T, KeyType>
where
    KeyType: StorageKey,
    T: Packed,
{
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        match self.get(index) {
            Some(value) => value,
            None => {
                panic!(
                    "index out of bounds: the len is {} but the index is {}",
                    self.len(),
                    index
                )
            }
        }
    }
}

impl<T, KeyType> core::ops::IndexMut<u32> for Vec<T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
{
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        let len = self.len();
        match self.get_mut(index) {
            Some(value) => value,
            None => {
                panic!(
                    "index out of bounds: the len is {} but the index is {}",
                    len, index
                )
            }
        }
    }
}

impl<'a, T: 'a, KeyType> IntoIterator for &'a Vec<T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T, KeyType>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: 'a, KeyType> IntoIterator for &'a mut Vec<T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
{
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T, KeyType>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, KeyType> Extend<T> for Vec<T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for item in iter {
            self.push(item)
        }
    }
}

impl<T, KeyType> FromIterator<T> for Vec<T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut vec = Vec::new();
        vec.extend(iter);
        vec
    }
}

impl<T, KeyType> core::cmp::PartialEq for Vec<T, KeyType>
where
    T: PartialEq + Packed,
    KeyType: StorageKey,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter().zip(other.iter()).all(|(lhs, rhs)| lhs == rhs)
    }
}

impl<T> core::cmp::Eq for Vec<T> where T: Eq + Packed {}

/// An iterator over shared references to the elements of a storage vector.
#[derive(Debug, Clone, Copy)]
pub struct Iter<'a, T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
{
    /// The storage vector to iterate over.
    vec: &'a Vec<T, KeyType>,
    /// The current begin of the iteration.
    begin: u32,
    /// The current end of the iteration.
    end: u32,
}

impl<'a, T, KeyType> Iter<'a, T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
{
    /// Creates a new iterator for the given storage vector.
    pub(crate) fn new(vec: &'a Vec<T, KeyType>) -> Self {
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

impl<'a, T, KeyType> Iterator for Iter<'a, T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
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
            return None;
        }
        let cur = self.begin + n;
        self.begin += 1 + n;
        self.vec.get(cur).expect("access is within bounds").into()
    }
}

impl<'a, T, KeyType> ExactSizeIterator for Iter<'a, T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
{
}

impl<'a, T, KeyType> DoubleEndedIterator for Iter<'a, T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        <Self as DoubleEndedIterator>::nth_back(self, 0)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        let n = n as u32;
        if self.begin >= self.end.saturating_sub(n) {
            return None;
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
pub struct IterMut<'a, T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
{
    /// The storage vector to iterate over.
    vec: &'a mut Vec<T, KeyType>,
    /// The current begin of the iteration.
    begin: u32,
    /// The current end of the iteration.
    end: u32,
}

impl<'a, T, KeyType> IterMut<'a, T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
{
    /// Creates a new iterator for the given storage vector.
    pub(crate) fn new(vec: &'a mut Vec<T, KeyType>) -> Self {
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

impl<'a, T, KeyType> IterMut<'a, T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
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

impl<'a, T, KeyType> Iterator for IterMut<'a, T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
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
            return None;
        }
        let cur = self.begin + n;
        self.begin += 1 + n;
        self.get_mut(cur).expect("access is within bounds").into()
    }
}

impl<'a, T, KeyType> ExactSizeIterator for IterMut<'a, T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
{
}

impl<'a, T, KeyType> DoubleEndedIterator for IterMut<'a, T, KeyType>
where
    T: Packed,
    KeyType: StorageKey,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        <Self as DoubleEndedIterator>::nth_back(self, 0)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        let n = n as u32;
        if self.begin >= self.end.saturating_sub(n) {
            return None;
        }
        self.end -= 1 + n;
        self.get_mut(self.end)
            .expect("access is within bounds")
            .into()
    }
}

#[cfg(test)]
mod tests {

    use crate::lazy::storage_vec::IndexOutOfBounds;

    use super::Vec as StorageVec;
    use core::cmp::Ordering;
    use ink_storage_traits::{ManualKey, Packed};

    #[test]
    fn new_vec_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // `StorageVec::new`
            let vec = <StorageVec<i32>>::new();
            assert!(vec.is_empty());
            assert_eq!(vec.len(), 0);
            assert_eq!(vec.get(0), None);
            assert!(vec.iter().next().is_none());
            // `StorageVec::default`
            let default = <StorageVec<i32> as Default>::default();
            assert!(default.is_empty());
            assert_eq!(default.len(), 0);
            assert_eq!(vec.get(0), None);
            assert!(default.iter().next().is_none());
            // `StorageVec::new` and `StorageVec::default` should be equal.
            assert_eq!(vec, default);
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn from_iterator_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let some_primes = [1, 2, 3, 5, 7, 11, 13];
            assert_eq!(some_primes.iter().copied().collect::<StorageVec<_>>(), {
                let mut vec = StorageVec::new();
                for prime in &some_primes {
                    vec.push(*prime)
                }
                vec
            });
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn from_empty_iterator_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            assert_eq!(
                [].iter().copied().collect::<StorageVec<i32>>(),
                StorageVec::new(),
            );
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn first_last_of_empty() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut vec = <StorageVec<u8>>::new();
            assert_eq!(vec.first(), None);
            assert_eq!(vec.first_mut(), None);
            assert_eq!(vec.last(), None);
            assert_eq!(vec.last_mut(), None);
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn push_pop_first_last_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            /// Asserts conditions are met for the given storage vector.
            fn assert_vec<F, L>(vec: &StorageVec<u8>, len: u32, first: F, last: L)
            where
                F: Into<Option<u8>>,
                L: Into<Option<u8>>,
            {
                assert_eq!(vec.is_empty(), len == 0);
                assert_eq!(vec.len(), len);
                assert_eq!(vec.first().copied(), first.into());
                assert_eq!(vec.last().copied(), last.into());
            }

            let mut vec = StorageVec::new();
            assert_vec(&vec, 0, None, None);

            // Sequence of `push`
            vec.push(b'a');
            assert_vec(&vec, 1, b'a', b'a');
            vec.push(b'b');
            assert_vec(&vec, 2, b'a', b'b');
            vec.push(b'c');
            assert_vec(&vec, 3, b'a', b'c');
            vec.push(b'd');
            assert_vec(&vec, 4, b'a', b'd');

            // Sequence of `pop`
            assert_eq!(vec.pop(), Some(b'd'));
            assert_vec(&vec, 3, b'a', b'c');
            assert_eq!(vec.pop(), Some(b'c'));
            assert_vec(&vec, 2, b'a', b'b');
            assert_eq!(vec.pop(), Some(b'b'));
            assert_vec(&vec, 1, b'a', b'a');
            assert_eq!(vec.pop(), Some(b'a'));
            assert_vec(&vec, 0, None, None);

            // Pop from empty vector.
            assert_eq!(vec.pop(), None);
            assert_vec(&vec, 0, None, None);
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn pop_drop_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let elems = [b'a', b'b', b'c', b'd'];
            let mut vec = vec_from_slice(&elems);
            assert_eq!(vec.pop_drop(), Some(()));
            assert_eq_slice(&vec, &elems[0..3]);
            assert_eq!(vec.pop_drop(), Some(()));
            assert_eq_slice(&vec, &elems[0..2]);
            assert_eq!(vec.pop_drop(), Some(()));
            assert_eq_slice(&vec, &elems[0..1]);
            assert_eq!(vec.pop_drop(), Some(()));
            assert_eq_slice(&vec, &[]);
            assert_eq!(vec.pop_drop(), None);
            assert_eq_slice(&vec, &[]);
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn get_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let elems = [b'a', b'b', b'c', b'd'];
            let mut vec = vec_from_slice(&elems);
            for (n, mut expected) in elems.iter().copied().enumerate() {
                let n = n as u32;
                assert_eq!(vec.get(n), Some(&expected));
                assert_eq!(vec.get_mut(n), Some(&mut expected));
                assert_eq!(&vec[n], &expected);
                assert_eq!(&mut vec[n], &mut expected);
            }
            let len = vec.len();
            assert_eq!(vec.get(len), None);
            assert_eq!(vec.get_mut(len), None);
            Ok(())
        })
        .unwrap()
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 3 but the index is 3")]
    fn index_out_of_bounds_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let test_values = [b'a', b'b', b'c'];
            let vec = vec_from_slice(&test_values);
            let _ = &vec[test_values.len() as u32];
            Ok(())
        })
        .unwrap()
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 3 but the index is 3")]
    fn index_mut_out_of_bounds_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let test_values = [b'a', b'b', b'c'];
            let mut vec = vec_from_slice(&test_values);
            let _ = &mut vec[test_values.len() as u32];
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn iter_next_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let elems = [b'a', b'b', b'c', b'd'];
            let vec = vec_from_slice(&elems);
            // Test iterator over `&T`:
            let mut iter = vec.iter();
            assert_eq!(iter.count(), 4);
            assert_eq!(iter.size_hint(), (4, Some(4)));
            assert_eq!(iter.next(), Some(&b'a'));
            assert_eq!(iter.size_hint(), (3, Some(3)));
            assert_eq!(iter.next(), Some(&b'b'));
            assert_eq!(iter.size_hint(), (2, Some(2)));
            assert_eq!(iter.count(), 2);
            assert_eq!(iter.next(), Some(&b'c'));
            assert_eq!(iter.size_hint(), (1, Some(1)));
            assert_eq!(iter.next(), Some(&b'd'));
            assert_eq!(iter.size_hint(), (0, Some(0)));
            assert_eq!(iter.count(), 0);
            assert_eq!(iter.next(), None);
            // Test iterator over `&mut T`:
            let mut vec = vec;
            let mut iter = vec.iter_mut();
            assert_eq!(iter.size_hint(), (4, Some(4)));
            assert_eq!(iter.next(), Some(&mut b'a'));
            assert_eq!(iter.size_hint(), (3, Some(3)));
            assert_eq!(iter.next(), Some(&mut b'b'));
            assert_eq!(iter.size_hint(), (2, Some(2)));
            assert_eq!(iter.next(), Some(&mut b'c'));
            assert_eq!(iter.size_hint(), (1, Some(1)));
            assert_eq!(iter.next(), Some(&mut b'd'));
            assert_eq!(iter.size_hint(), (0, Some(0)));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.count(), 0);
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn iter_nth_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let elems = [b'a', b'b', b'c', b'd'];
            let vec = vec_from_slice(&elems);
            // Test iterator over `&T`:
            let mut iter = vec.iter();
            assert_eq!(iter.count(), 4);
            assert_eq!(iter.size_hint(), (4, Some(4)));
            assert_eq!(iter.nth(1), Some(&b'b'));
            assert_eq!(iter.count(), 2);
            assert_eq!(iter.size_hint(), (2, Some(2)));
            assert_eq!(iter.nth(1), Some(&b'd'));
            assert_eq!(iter.size_hint(), (0, Some(0)));
            assert_eq!(iter.count(), 0);
            assert_eq!(iter.nth(1), None);
            // Test iterator over `&mut T`:
            let mut vec = vec;
            let mut iter = vec.iter_mut();
            assert_eq!(iter.size_hint(), (4, Some(4)));
            assert_eq!(iter.nth(1), Some(&mut b'b'));
            assert_eq!(iter.size_hint(), (2, Some(2)));
            assert_eq!(iter.nth(1), Some(&mut b'd'));
            assert_eq!(iter.size_hint(), (0, Some(0)));
            assert_eq!(iter.nth(1), None);
            assert_eq!(iter.count(), 0);
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn iter_next_back_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let elems = [b'a', b'b', b'c', b'd'];
            let vec = vec_from_slice(&elems);
            // Test iterator over `&T`:
            let mut iter = vec.iter().rev();
            assert_eq!(iter.clone().count(), 4);
            assert_eq!(iter.next(), Some(&b'd'));
            assert_eq!(iter.next(), Some(&b'c'));
            assert_eq!(iter.clone().count(), 2);
            assert_eq!(iter.next(), Some(&b'b'));
            assert_eq!(iter.next(), Some(&b'a'));
            assert_eq!(iter.clone().count(), 0);
            assert_eq!(iter.next(), None);
            // Test iterator over `&mut T`:
            let mut vec = vec;
            let mut iter = vec.iter_mut().rev();
            assert_eq!(iter.next(), Some(&mut b'd'));
            assert_eq!(iter.next(), Some(&mut b'c'));
            assert_eq!(iter.next(), Some(&mut b'b'));
            assert_eq!(iter.next(), Some(&mut b'a'));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.count(), 0);
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn iter_nth_back_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let elems = [b'a', b'b', b'c', b'd'];
            let vec = vec_from_slice(&elems);
            // Test iterator over `&T`:
            let mut iter = vec.iter().rev();
            assert_eq!(iter.clone().count(), 4);
            assert_eq!(iter.nth(1), Some(&b'c'));
            assert_eq!(iter.clone().count(), 2);
            assert_eq!(iter.nth(1), Some(&b'a'));
            assert_eq!(iter.clone().count(), 0);
            assert_eq!(iter.nth(1), None);
            // Test iterator over `&mut T`:
            let mut vec = vec;
            let mut iter = vec.iter_mut().rev();
            assert_eq!(iter.nth(1), Some(&mut b'c'));
            assert_eq!(iter.nth(1), Some(&mut b'a'));
            assert_eq!(iter.nth(1), None);
            assert_eq!(iter.count(), 0);
            Ok(())
        })
        .unwrap()
    }

    /// Asserts that the given ordered storage vector elements are equal to the
    /// ordered elements of the given slice.
    fn assert_eq_slice(vec: &StorageVec<u8>, slice: &[u8]) {
        assert!(vec.iter().zip(slice.iter()).all(|(lhs, rhs)| *lhs == *rhs))
    }

    /// Creates a storage vector from the given slice.
    fn vec_from_slice<T: Copy + Packed>(slice: &[T]) -> StorageVec<T> {
        slice.iter().copied().collect::<StorageVec<T>>()
    }

    #[test]
    fn swap_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let elems = [b'a', b'b', b'c', b'd'];
            let mut vec = vec_from_slice(&elems);

            // Swap at same position is a no-op.
            for index in 0..elems.len() as u32 {
                vec.swap(index, index);
                assert_eq_slice(&vec, &elems);
            }

            // Swap first and second
            vec.swap(0, 1);
            assert_eq_slice(&vec, &[b'b', b'a', b'c', b'd']);
            // Swap third and last
            vec.swap(2, 3);
            assert_eq_slice(&vec, &[b'b', b'a', b'd', b'c']);
            // Swap first and last
            vec.swap(0, 3);
            assert_eq_slice(&vec, &[b'c', b'a', b'd', b'b']);
            Ok(())
        })
        .unwrap()
    }

    #[test]
    #[should_panic]
    fn swap_one_invalid_index() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut vec = vec_from_slice(&[b'a', b'b', b'c', b'd']);
            vec.swap(0, vec.len());
            Ok(())
        })
        .unwrap()
    }

    #[test]
    #[should_panic]
    fn swap_both_invalid_indices() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut vec = vec_from_slice(&[b'a', b'b', b'c', b'd']);
            vec.swap(vec.len(), vec.len());
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn swap_remove_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut vec = vec_from_slice(&[b'a', b'b', b'c', b'd']);

            // Swap remove first element.
            assert_eq!(vec.swap_remove(0), Some(b'a'));
            assert_eq_slice(&vec, &[b'd', b'b', b'c']);
            // Swap remove middle element.
            assert_eq!(vec.swap_remove(1), Some(b'b'));
            assert_eq_slice(&vec, &[b'd', b'c']);
            // Swap remove last element.
            assert_eq!(vec.swap_remove(1), Some(b'c'));
            assert_eq_slice(&vec, &[b'd']);
            // Swap remove only element.
            assert_eq!(vec.swap_remove(0), Some(b'd'));
            assert_eq_slice(&vec, &[]);
            // Swap remove from empty vector.
            assert_eq!(vec.swap_remove(0), None);
            assert_eq_slice(&vec, &[]);
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn swap_remove_drop_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut vec = vec_from_slice(&[b'a', b'b', b'c', b'd']);

            // Swap remove first element.
            assert_eq!(vec.swap_remove_drop(0), Some(()));
            assert_eq_slice(&vec, &[b'd', b'b', b'c']);
            // Swap remove middle element.
            assert_eq!(vec.swap_remove_drop(1), Some(()));
            assert_eq_slice(&vec, &[b'd', b'c']);
            // Swap remove last element.
            assert_eq!(vec.swap_remove_drop(1), Some(()));
            assert_eq_slice(&vec, &[b'd']);
            // Swap remove only element.
            assert_eq!(vec.swap_remove_drop(0), Some(()));
            assert_eq_slice(&vec, &[]);
            // Swap remove from empty vector.
            assert_eq!(vec.swap_remove_drop(0), None);
            assert_eq_slice(&vec, &[]);
            Ok(())
        })
        .unwrap();
    }

    /*
    #[test]
    #[should_panic(expected = "encountered empty storage cell")]
    fn spread_layout_clear_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let vec1 = vec_from_slice(&[b'a', b'b', b'c', b'd']);
            let root_key = Key::from([0x42; 32]);
            SpreadLayout::push_spread(&vec1, &mut KeyPtr::from(root_key));
            // It has already been asserted that a valid instance can be pulled
            // from contract storage after a push to the same storage region.
            //
            // Now clear the associated storage from `vec1` and check whether
            // loading another instance from this storage will panic since the
            // vector's length property cannot read a value:
            SpreadLayout::clear_spread(&vec1, &mut KeyPtr::from(root_key));
            let _ = <StorageVec<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(
                root_key,
            ));
            Ok(())
        })
        .unwrap()
    }
    */

    #[test]
    fn set_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut vec = vec_from_slice(&[b'a', b'b', b'c', b'd']);
            vec.set(0, b'x').unwrap();
            let expected = vec_from_slice(&[b'x', b'b', b'c', b'd']);
            assert_eq!(vec, expected);
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn set_fails_when_index_oob() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut vec = vec_from_slice(&[b'a']);
            let res = vec.set(1, b'x');
            assert_eq!(res, Err(IndexOutOfBounds));
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn clear_works_on_filled_vec() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut vec = vec_from_slice(&[b'a', b'b', b'c', b'd']);
            vec.clear();
            assert!(vec.is_empty());
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn clear_works_on_empty_vec() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut vec: StorageVec<()> = vec_from_slice(&[]);
            vec.clear();
            assert!(vec.is_empty());
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn test_binary_search() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let b: StorageVec<i32> = StorageVec::new();
            assert_eq!(b.binary_search(&5), Err(0));

            let b = vec_from_slice(&[4]);
            assert_eq!(b.binary_search(&3), Err(0));
            assert_eq!(b.binary_search(&4), Ok(0));
            assert_eq!(b.binary_search(&5), Err(1));

            let b = vec_from_slice(&[1, 2, 4, 6, 8, 9]);
            dbg!(b.len());
            dbg!(&b);
            assert_eq!(b.binary_search(&5), Err(3));
            assert_eq!(b.binary_search(&6), Ok(3));
            assert_eq!(b.binary_search(&7), Err(4));
            assert_eq!(b.binary_search(&8), Ok(4));

            let b = vec_from_slice(&[1, 2, 4, 5, 6, 8]);
            assert_eq!(b.binary_search(&9), Err(6));

            let b = vec_from_slice(&[1, 2, 4, 6, 7, 8, 9]);
            assert_eq!(b.binary_search(&6), Ok(3));
            assert_eq!(b.binary_search(&5), Err(3));
            assert_eq!(b.binary_search(&8), Ok(5));

            let b = vec_from_slice(&[1, 2, 4, 5, 6, 8, 9]);
            assert_eq!(b.binary_search(&7), Err(5));
            assert_eq!(b.binary_search(&0), Err(0));

            let b = vec_from_slice(&[1, 3, 3, 3, 7]);
            assert_eq!(b.binary_search(&0), Err(0));
            assert_eq!(b.binary_search(&1), Ok(0));
            assert_eq!(b.binary_search(&2), Err(1));
            matches!(b.binary_search(&3), Ok(1..=3));
            assert_eq!(b.binary_search(&4), Err(4));
            assert_eq!(b.binary_search(&5), Err(4));
            assert_eq!(b.binary_search(&6), Err(4));
            assert_eq!(b.binary_search(&7), Ok(4));
            assert_eq!(b.binary_search(&8), Err(5));

            let b = vec_from_slice(&[(); u8::MAX as usize]);
            assert_eq!(b.binary_search(&()), Ok(u8::MAX as u32 / 2));
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn test_binary_search_by_overflow() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let b = vec_from_slice(&[(); u8::MAX as usize]);
            assert_eq!(
                b.binary_search_by(|_| Ordering::Equal),
                Ok(u8::MAX as u32 / 2)
            );
            assert_eq!(b.binary_search_by(|_| Ordering::Greater), Err(0));
            assert_eq!(b.binary_search_by(|_| Ordering::Less), Err(u8::MAX as u32));
            Ok(())
        })
        .unwrap()
    }

    #[test]
    // Test implementation specific behavior when finding equivalent elements.
    fn test_binary_search_implementation_details() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let b = vec_from_slice(&[1, 1, 2, 2, 3, 3, 3]);
            assert_eq!(b.binary_search(&1), Ok(1));
            assert_eq!(b.binary_search(&2), Ok(3));
            assert_eq!(b.binary_search(&3), Ok(5));
            let b = vec_from_slice(&[1, 1, 1, 1, 1, 3, 3, 3, 3]);
            assert_eq!(b.binary_search(&1), Ok(4));
            assert_eq!(b.binary_search(&3), Ok(7));
            let b = vec_from_slice(&[1, 1, 1, 1, 3, 3, 3, 3, 3]);
            assert_eq!(b.binary_search(&1), Ok(2));
            assert_eq!(b.binary_search(&3), Ok(4));
            Ok(())
        })
        .unwrap()
    }

    /*
    #[test]
    #[should_panic(expected = "encountered empty storage cell")]
    fn storage_is_cleared_completely_after_pull_lazy() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // given
            let root_key = Key::from([0x42; 32]);
            let mut lazy_vec: Lazy<StorageVec<u32>> = Lazy::new(StorageVec::new());
            lazy_vec.push(13u32);
            lazy_vec.push(13u32);
            SpreadLayout::push_spread(&lazy_vec, &mut KeyPtr::from(root_key));
            let pulled_vec = <Lazy<StorageVec<u32>> as SpreadLayout>::pull_spread(
                &mut KeyPtr::from(root_key),
            );

            // when
            SpreadLayout::clear_spread(&pulled_vec, &mut KeyPtr::from(root_key));

            // then
            let contract_id = ink_env::test::callee::<ink_env::DefaultEnvironment>();
            let used_cells = ink_env::test::count_used_storage_cells::<
                ink_env::DefaultEnvironment,
            >(&contract_id)
            .expect("used cells must be returned");
            assert_eq!(used_cells, 0);
            let _ = *<Lazy<Lazy<u32>> as SpreadLayout>::pull_spread(&mut KeyPtr::from(
                root_key,
            ));

            Ok(())
        })
        .unwrap()
    }
    */

    #[test]
    //#[should_panic(expected = "encountered empty storage cell")]
    fn drop_works() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // if the setup panics it should not cause the test to pass
            let setup_result = std::panic::catch_unwind(|| {
                vec_from_slice(&[b'a', b'b', b'c', b'd']).write();

                // vec is dropped which should clear the cells
            });
            assert!(setup_result.is_ok(), "setup should not panic");

            let contract_id = ink_env::test::callee::<ink_env::DefaultEnvironment>();
            let used_cells = ink_env::test::count_used_storage_cells::<
                ink_env::DefaultEnvironment,
            >(&contract_id)
            .expect("used cells must be returned");
            assert_eq!(used_cells, 0);

            //let _ = <StorageVec<u8> as SpreadLayout>::pull_spread(&mut KeyPtr::from(
            //    root_key,
            //));
            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn write_work() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let a = [1, 1, 2, 2, 3, 3, 3]
                .iter()
                .copied()
                .collect::<StorageVec<i32, ManualKey<1>>>();
            let b = StorageVec::<i32, ManualKey<1>>::new();

            a.write();

            assert_eq!(b.len(), 7);
            assert_eq!(a, b);

            Ok(())
        })
        .unwrap()
    }
}
