// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

use crate::storage::{
	self,
	Key,
	chunk::SyncChunk,
	Allocator,
	Flush,
};

use parity_codec::{Encode, Decode};

/// A stash collection.
///
/// Provides O(1) random insertion, deletion and access of its elements.
///
/// # Details
///
/// An `O(1)` amortized table that reuses keys.
///
/// ## Guarantees and non-guarantees:
///
/// 1. `Stash` is deterministic and keys do not depend on the inserted values.
///    This means you can update two stashes in tandem and get the same keys
///    back. This could be useful for, e.g., primary/secondary replication.
/// 2. Keys will always be less than the maximum number of items that have ever
///    been present in the `Stash` at any single point in time. In other words,
///    if you never store more than `n` items in a `Stash`, the stash will only
///    assign keys less than `n`. You can take advantage of this guarantee to
///    truncate the key from a `usize` to some smaller type.
/// 3. Except the guarantees noted above, you can assume nothing about key
///    assignment or iteration order. They can change at any time.
#[derive(Debug)]
pub struct Stash<T> {
	/// The latest vacant index.
	next_vacant: storage::Value<u32>,
	/// The number of items stored in the stash.
	///
	/// # Note
	///
	/// We cannot simply use the underlying length of the vector
	/// since it would include vacant slots as well.
	len: storage::Value<u32>,
	/// The maximum length the stash ever had.
	max_len: storage::Value<u32>,
	/// The entries of the stash.
	entries: SyncChunk<Entry<T>>,
}

/// Iterator over the values of a stash.
#[derive(Debug)]
pub struct Values<'a, T> {
	/// The underlying iterator.
	iter: Iter<'a, T>,
}

impl<'a, T> Values<'a, T> {
	/// Creates a new iterator for the given storage stash.
	pub(crate) fn new(stash: &'a Stash<T>) -> Self {
		Self{iter: stash.iter()}
	}
}

impl<T> Flush for Stash<T>
where
	T: parity_codec::Encode,
{
	fn flush(&mut self) {
		self.next_vacant.flush();
		self.len.flush();
		self.max_len.flush();
		self.entries.flush();
	}
}

impl<'a, T> Iterator for Values<'a, T>
where
	T: parity_codec::Codec
{
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|(_index, value)| value)
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.iter.size_hint()
	}
}

impl<'a, T> ExactSizeIterator for Values<'a, T>
where
	T: parity_codec::Codec
{}

impl<'a, T> DoubleEndedIterator for Values<'a, T>
where
	T: parity_codec::Codec
{
	fn next_back(&mut self) -> Option<Self::Item> {
		self.iter.next_back().map(|(_index, value)| value)
	}
}

/// Iterator over the entries of a stash.
#[derive(Debug)]
pub struct Iter<'a, T> {
	/// The stash that is iterated over.
	stash: &'a Stash<T>,
	/// The index of the current start item of the iteration.
	begin: u32,
	/// The index of the current end item of the iteration.
	end: u32,
	/// The amount of already yielded items.
	///
	/// Required to offer an exact `size_hint` implementation.
	/// Also can be used to exit iteration as early as possible.
	yielded: u32,
}

impl<'a, T> Iter<'a, T> {
	/// Creates a new iterator for the given storage stash.
	pub(crate) fn new(stash: &'a Stash<T>) -> Self {
		Self{
			stash,
			begin: 0,
			end: stash.max_len(),
			yielded: 0,
		}
	}
}

impl<'a, T> Iterator for Iter<'a, T>
where
	T: parity_codec::Codec
{
	type Item = (u32, &'a T);

	fn next(&mut self) -> Option<Self::Item> {
		debug_assert!(self.begin <= self.end);
		if self.yielded == self.stash.len() {
			return None
		}
		while self.begin < self.end {
			let cur = self.begin;
			self.begin += 1;
			if let Some(elem) = self.stash.get(cur) {
				self.yielded += 1;
				return Some((cur, elem))
			}
		}
		None
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let remaining = (self.stash.len() - self.yielded) as usize;
		(remaining, Some(remaining))
	}
}

impl<'a, T> ExactSizeIterator for Iter<'a, T>
where
	T: parity_codec::Codec
{}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
	T: parity_codec::Codec
{
	fn next_back(&mut self) -> Option<Self::Item> {
		debug_assert!(self.begin <= self.end);
		if self.yielded == self.stash.len() {
			return None
		}
		while self.begin < self.end {
			self.end -= 1;
			if let Some(elem) = self.stash.get(self.end) {
				self.yielded += 1;
				return Some((self.end, elem))
			}
		}
		None
	}
}

/// An entry within a stash collection.
///
/// This represents either an occupied entry with its associated value
/// or a vacant entry pointing to the next vacant entry.
#[derive(Debug)]
#[derive(Encode, Decode)]
enum Entry<T> {
	/// A vacant entry pointing to the next vacant index.
	Vacant(u32),
	/// An occupied entry containing the value.
	Occupied(T),
}

impl<T> parity_codec::Encode for Stash<T> {
	fn encode_to<W: parity_codec::Output>(&self, dest: &mut W) {
		self.next_vacant.encode_to(dest);
		self.len.encode_to(dest);
		self.max_len.encode_to(dest);
		self.entries.encode_to(dest);
	}
}

impl<T> parity_codec::Decode for Stash<T> {
	fn decode<I: parity_codec::Input>(input: &mut I) -> Option<Self> {
		let next_vacant = storage::Value::decode(input)?;
		let len = storage::Value::decode(input)?;
		let max_len = storage::Value::decode(input)?;
		let entries = SyncChunk::decode(input)?;
		Some(Self{next_vacant, len, max_len, entries})
	}
}

impl<T> Stash<T> {
	/// Allocates a new storage stash using the given storage allocator.
	///
	/// # Safety
	///
	/// The is unsafe because it does not check if the associated storage
	/// does not alias with storage allocated by other storage allocators.
	pub unsafe fn new_using_alloc<A>(alloc: &mut A) -> Self
	where
		A: Allocator
	{
		Self{
			next_vacant: storage::Value::new_using_alloc(alloc, 0),
			len: storage::Value::new_using_alloc(alloc, 0),
			max_len: storage::Value::new_using_alloc(alloc, 0),
			entries: SyncChunk::new_using_alloc(alloc),
		}
	}

	/// Returns an iterator over the references of all entries of the stash.
	///
	/// # Note
	///
	/// - It is **not** recommended to iterate over all elements of a storage stash.
	/// - Try to avoid this if possible or iterate only over a minimal subset of
	///   all elements using e.g. `Iterator::take(n)`.
	pub fn iter(&self) -> Iter<T> {
		Iter::new(self)
	}

	/// Returns an iterator over the references of all values of the stash.
	///
	/// # Note
	///
	/// - It is **not** recommended to iterate over all elements of a storage stash.
	/// - Try to avoid this if possible or iterate only over a minimal subset of
	///   all elements using e.g. `Iterator::take(n)`.
	pub fn values(&self) -> Values<T> {
		Values::new(self)
	}

	/// Returns the unterlying key to the cells.
	///
	/// # Note
	///
	/// This is a low-level utility getter and should
	/// normally not be required by users.
	pub fn entries_key(&self) -> Key {
		self.entries.cells_key()
	}

	/// Returns the number of elements stored in the stash.
	pub fn len(&self) -> u32 {
		*self.len.get()
	}

	/// Returns the maximum number of element stored in the
	/// stash at the same time.
	pub fn max_len(&self) -> u32 {
		*self.max_len.get()
	}

	/// Returns `true` if the stash contains no elements.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns the next vacant index.
	fn next_vacant(&self) -> u32 {
		*self.next_vacant.get()
	}

}

impl<T> Stash<T>
where
	T: parity_codec::Codec,
{
	/// Returns the element stored at index `n` if any.
	pub fn get(&self, n: u32) -> Option<&T> {
		self
			.entries
			.get(n)
			.and_then(|entry| match entry {
				Entry::Occupied(val) => Some(val),
				Entry::Vacant(_) => None,
			})
	}

	/// Put the element into the stash at the next vacant position.
	///
	/// Returns the stash index that the element was put into.
	pub fn put(&mut self, val: T) -> u32 {
		let current_vacant = *self
			.next_vacant
			.get();
		debug_assert!(current_vacant <= self.len());
		if current_vacant == self.len() {
			self.entries.set(current_vacant, Entry::Occupied(val));
			self.next_vacant.set(current_vacant + 1);
			self.max_len.set(self.max_len() + 1);
		} else {
			let next_vacant = match
				self
					.entries
					.put(current_vacant, Entry::Occupied(val))
					.expect(
						"[pdsl_core::Stash::put] Error: \
						expected a vacant entry here, but no entry was found"
					)
				{
					Entry::Vacant(next_vacant) => next_vacant,
					Entry::Occupied(_) => unreachable!(
						"[pdsl_core::Stash::put] Error: \
						a next_vacant index can never point to an occupied entry"
					)
				};
			self.next_vacant.set(next_vacant);
		}
		self.len.set(self.len() + 1);
		current_vacant
	}

	/// Takes the element stored at index `n`-th if any.
	pub fn take(&mut self, n: u32) -> Option<T> {
		match self.entries.get(n) {
			| None
			| Some(Entry::Vacant(_)) => None,
			| Some(Entry::Occupied(_)) => {
				match self.entries.put(n, Entry::Vacant(self.next_vacant())).expect(
					"[pdsl_core::Stash::take] Error: \
					 we already asserted that the entry at `n` exists"
				) {
					Entry::Occupied(val) => {
						self.next_vacant.set(n);
						debug_assert!(self.len() >= 1);
						self.len.set(self.len() - 1);
						Some(val)
					},
					Entry::Vacant(_) => unreachable!(
						"[pdsl_core::Stash::take] Error: \
						 we already asserted that the entry is occupied"
					)
				}
			}
		}
	}
}
