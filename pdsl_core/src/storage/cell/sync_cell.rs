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
		cell::TypedCell,
		Allocator,
	},
};

use core::cell::RefCell;

/// A synchronized cell.
///
/// Provides interpreted, read-optimized and inplace-mutable
/// access to the associated constract storage slot.
///
/// # Guarantees
///
/// - `Owned`, `Typed`, `Avoid Reads`, `Mutable`
///
/// Read more about kinds of guarantees and their effect [here](../index.html#guarantees).
#[derive(Debug)]
pub struct SyncCell<T> {
	/// The underlying typed cell.
	cell: TypedCell<T>,
	/// The cache for the synchronized value.
	cache: Cache<T>,
}

/// A cache entry storing the value if synchronized.
#[derive(Debug)]
pub enum CacheEntry<T> {
	/// The cache is desychronized with the contract storage.
	Desync,
	/// The cache is in sync with the contract storage.
	Sync(Option<T>),
}

#[derive(Debug)]
pub struct Cache<T> {
	/// The cached value.
	entry: RefCell<CacheEntry<T>>,
}

impl<T> Default for Cache<T> {
	fn default() -> Self {
		Self{ entry: RefCell::new(CacheEntry::Desync) }
	}
}

impl<T> CacheEntry<T> {
	/// Returns `true` if the cache is in sync.
	pub fn is_synced(&self) -> bool {
		match self {
			CacheEntry::Sync(_) => true,
			_ => false,
		}
	}

	/// Unwraps an immutable reference to the synchronized value.
	///
	/// # Panics
	///
	/// Panics if the cache is in a desynchronized state.
	pub fn unwrap_get(&self) -> Option<&T> {
		match self {
			CacheEntry::Sync(opt_elem) => opt_elem.into(),
			CacheEntry::Desync => {
				panic!(
					"[pdsl_core::sync_cell::CacheEntry::unwrap] Error: \
					 tried to unwrap a desynchronized value"
				)
			}
		}
	}
}

impl<T> Cache<T> {
	/// Returns `true` if the cache is in sync.
	pub fn is_synced(&self) -> bool {
		self.entry.borrow().is_synced()
	}

	/// Updates the synchronized value.
	///
	/// # Note
	///
	/// The cache will be in sync after this operation.
	pub fn update(&self, new_val: Option<T>) {
		self.entry.replace(
			CacheEntry::Sync(new_val)
		);
	}

	/// Returns an immutable reference to the cache entry.
	pub fn get(&self) -> &CacheEntry<T> {
		unsafe{ &*self.entry.as_ptr() }
	}

	/// Mutates the synchronized value if any.
	///
	/// Returns an immutable reference to the result if
	/// a mutation happened, otherwise `None` is returned.
	pub fn mutate_with<F>(&mut self, f: F) -> Option<&T>
	where
		F: FnOnce(&mut T)
	{
		match self.entry.get_mut() {
			CacheEntry::Desync => None,
			CacheEntry::Sync(opt_val) => {
				if let Some(val) = opt_val {
					f(val);
					Some(&*val)
				} else {
					None
				}
			}
		}
	}
}

impl<T> parity_codec::Encode for SyncCell<T> {
	fn encode_to<W: parity_codec::Output>(&self, dest: &mut W) {
		self.cell.encode_to(dest)
	}
}

impl<T> parity_codec::Decode for SyncCell<T> {
	fn decode<I: parity_codec::Input>(input: &mut I) -> Option<Self> {
		TypedCell::decode(input)
			.map(|typed_cell| Self{
				cell: typed_cell,
				cache: Cache::default()
			})
	}
}

impl<T> SyncCell<T> {
	/// Creates a new copy cell for the given key.
	///
	/// # Safety
	///
	/// This is unsafe since it does not check if the associated
	/// contract storage does not alias with other accesses.
	pub unsafe fn new_unchecked(key: Key) -> Self {
		Self{
			cell: TypedCell::new_unchecked(key),
			cache: Default::default(),
		}
	}

	/// Allocates a new sync cell using the given storage allocator.
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
			cell: TypedCell::new_using_alloc(alloc),
			cache: Default::default(),
		}
	}

	/// Removes the value from the cell.
	pub fn clear(&mut self) {
		self.cell.clear();
		self.cache.update(None);
	}
}

impl<T> SyncCell<T>
where
	T: parity_codec::Decode
{
	/// Returns the value of the cell if any.
	pub fn get(&self) -> Option<&T> {
		match self.cache.get() {
			CacheEntry::Desync => {
				self.load()
			}
			CacheEntry::Sync(opt_elem) => {
				opt_elem.into()
			}
		}
	}

	/// Returns an immutable reference to the entity if any.
	///
	/// # Note
	///
	/// Prefer using [`get`](struct.SyncCell.html#method.get)
	/// to avoid unnecesary contract storage accesses.
	fn load(&self) -> Option<&T> {
		self.cache.update(self.cell.load());
		// Now cache is certainly synchronized
		// so we can safely unwrap the cached value.
		debug_assert!(self.cache.is_synced());
		self.cache.get().unwrap_get()
	}
}

impl<T> SyncCell<T>
where
	T: parity_codec::Encode
{
	/// Sets the value of the cell.
	pub fn set(&mut self, val: T) {
		self.cell.store(&val);
		self.cache.update(Some(val))
	}
}

impl<T> SyncCell<T>
where
	T: parity_codec::Codec
{
	/// Mutates the value stored in the cell.
	///
	/// Returns an immutable reference to the result if
	/// a mutation happened, otherwise `None` is returned.
	pub fn mutate_with<F>(&mut self, f: F) -> Option<&T>
	where
		F: FnOnce(&mut T)
	{
		if !self.cache.is_synced() {
			self.load();
		}
		debug_assert!(self.cache.is_synced());
		match self.cache.mutate_with(f) {
			Some(res) => {
				self.cell.store(res);
				Some(&*res)
			}
			None => None
		}
	}
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
	use super::*;

	use crate::env::TestEnv;

	#[test]
	fn simple() {
		let mut cell: SyncCell<i32> = unsafe {
			SyncCell::new_unchecked(Key([0x42; 32]))
		};
		assert_eq!(cell.get(), None);
		cell.set(5);
		assert_eq!(cell.get(), Some(&5));
		assert_eq!(cell.mutate_with(|val| *val += 10), Some(&15));
		assert_eq!(cell.get(), Some(&15));
		cell.clear();
		assert_eq!(cell.get(), None);
	}

	#[test]
	fn count_reads() {
		let cell: SyncCell<i32> = unsafe {
			SyncCell::new_unchecked(Key([0x42; 32]))
		};
		assert_eq!(TestEnv::total_reads(), 0);
		cell.get();
		assert_eq!(TestEnv::total_reads(), 1);
		cell.get();
		cell.get();
		assert_eq!(TestEnv::total_reads(), 1);
	}

	#[test]
	fn count_writes() {
		let mut cell: SyncCell<i32> = unsafe {
			SyncCell::new_unchecked(Key([0x42; 32]))
		};
		assert_eq!(TestEnv::total_writes(), 0);
		cell.set(1);
		assert_eq!(TestEnv::total_writes(), 1);
		cell.set(2);
		cell.set(3);
		assert_eq!(TestEnv::total_writes(), 3);
	}
}
