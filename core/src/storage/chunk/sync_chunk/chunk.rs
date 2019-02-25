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

use crate::{
	storage::{
		Key,
		chunk::{
			TypedChunk,
		},
		alloc::{
			Allocate,
			AllocateUsing,
		},
		Flush,
	},
};
use super::CacheGuard;

/// A chunk of synchronized cells.
///
/// Provides mutable and read-optimized access to the associated constract storage slot.
///
/// # Guarantees
///
/// - `Owned`
/// - `Typed`
/// - `Opt. Reads`
/// - `Mutable`
///
/// Read more about kinds of guarantees and their effect [here](../index.html#guarantees).
#[derive(Debug)]
pub struct SyncChunk<T> {
	/// The underlying chunk of cells.
	chunk: TypedChunk<T>,
	/// The cached element.
	cache: CacheGuard<T>,
}

impl<T> Flush for SyncChunk<T>
where
	T: parity_codec::Encode,
{
	fn flush(&mut self) {
		for (n, dirty_val) in self.cache.iter_dirty() {
			match dirty_val.get() {
				Some(val) => self.chunk.store(n, val),
				None => self.chunk.clear(n),
			}
			dirty_val.mark_clean();
		}
	}
}

impl<T> parity_codec::Encode for SyncChunk<T> {
	fn encode_to<W: parity_codec::Output>(&self, dest: &mut W) {
		self.chunk.encode_to(dest)
	}
}

impl<T> parity_codec::Decode for SyncChunk<T> {
	fn decode<I: parity_codec::Input>(input: &mut I) -> Option<Self> {
		TypedChunk::decode(input)
			.map(|typed_chunk| Self{
				chunk: typed_chunk,
				cache: Default::default(),
			})
	}
}

impl<T> AllocateUsing for SyncChunk<T> {
	unsafe fn allocate_using<A>(alloc: &mut A) -> Self
	where
		A: Allocate,
	{
		Self{
			chunk: TypedChunk::allocate_using(alloc),
			cache: CacheGuard::default(),
		}
	}
}

impl<T> SyncChunk<T> {
	/// Clears the cache value at position `n`.
	pub fn clear(&mut self, n: u32) {
		self.cache.update_mut(n, None);
	}

	/// Returns the unterlying key to the cells.
	///
	/// # Note
	///
	/// This is a low-level utility getter and should
	/// normally not be required by users.
	pub fn cells_key(&self) -> Key {
		self.chunk.cells_key()
	}
}

impl<T> SyncChunk<T>
where
	T: parity_codec::Decode
{
	/// Returns the value of the `n`-th cell if any.
	#[must_use]
	pub fn get(&self, n: u32) -> Option<&T> {
		match self.cache.get(n) {
			Some(cache_value) => cache_value.get(),
			None => {
				self.cache.update(n, self.chunk.load(n))
			}
		}
	}

	/// Returns the value of the `n`-th cell if any.
	#[must_use]
	pub fn get_mut(&mut self, n: u32) -> Option<&mut T> {
		match self.cache.get_mut(n) {
			Some(cache_value) => cache_value.get_mut(),
			None => {
				self.cache.update_mut(n, self.chunk.load(n))
			}
		}
	}

	/// Takes the value of the `n`-th cell if any.
	///
	/// # Note
	///
	/// Prefer using [clear](struct.SyncChunk.html#method.clear)
	/// if you are not interested in the return value since it
	/// is more efficient.
	#[must_use]
	pub fn take(&mut self, n: u32) -> Option<T> {
		match self.cache.get_mut(n) {
			Some(cache_value) => cache_value.take(),
			None => {
				self.cache.update_mut(n, None);
				self.chunk.load(n)
			}
		}
	}
}

impl<T> SyncChunk<T>
where
	T: parity_codec::Encode
{
	/// Sets the value of the `n`-th cell.
	pub fn set(&mut self, n: u32, val: T) {
		self.cache.update_mut(n, Some(val));
	}
}

impl<T> SyncChunk<T>
where
	T: parity_codec::Codec
{
	/// Replaces the value of the `n`-th cell and returns its old value if any.
	///
	/// # Note
	///
	/// Prefer using [set](struct.SyncChunk.html#method.set)
	/// if you are not interested in the return value since it
	/// is more efficient.
	#[must_use]
	pub fn put(&mut self, n: u32, new_val: T) -> Option<T> {
		match self.cache.get_mut(n) {
			Some(cache_value) => cache_value.put(Some(new_val)),
			None => {
				self.cache.update_mut(n, Some(new_val));
				self.chunk.load(n)
			}
		}
	}
}
