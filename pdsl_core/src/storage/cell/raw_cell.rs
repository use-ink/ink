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
	memory::vec::Vec,
	storage::{
		Key,
		NonCloneMarker,
		Allocator,
	},
	env::{Env, ContractEnv},
};

use parity_codec::{Encode, Decode};

/// A raw cell.
///
/// Provides uninterpreted and unformatted access to the associated contract storage slot.
///
/// # Guarantees
///
/// - `Owned`
///
/// Read more about kinds of guarantees and their effect [here](../index.html#guarantees).
#[derive(Debug, PartialEq, Eq, Hash, Encode, Decode)]
pub struct RawCell {
	/// The key to the associated constract storage slot.
	key: Key,
	/// Marker that prevents this type from being `Copy` or `Clone` by accident.
	non_clone: NonCloneMarker<()>,
}

impl RawCell {
	/// Creates a new raw cell for the given key.
	///
	/// # Safety
	///
	/// This is unsafe since it does not check if the associated
	/// contract storage does not alias with other accesses.
	pub unsafe fn new_unchecked(key: Key) -> Self {
		Self{
			key: key,
			non_clone: NonCloneMarker::default()
		}
	}

	/// Allocates a new raw cell using the given storage allocator.
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
			key: alloc.alloc(1),
			non_clone: Default::default(),
		}
	}
}

impl RawCell {
	/// Loads the bytes stored in the cell if not empty.
	pub fn load(&self) -> Option<Vec<u8>> {
		unsafe { ContractEnv::load(self.key) }
	}

	/// Stores the given bytes into the cell.
	pub fn store(&mut self, bytes: &[u8]) {
		unsafe { ContractEnv::store(self.key, bytes) }
	}

	/// Removes the bytes stored in the cell.
	pub fn clear(&mut self) {
		unsafe { ContractEnv::clear(self.key) }
	}
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
	use super::*;

	use crate::{
		test_utils::run_test,
		env::TestEnv,
	};

	#[test]
	fn simple() {
		run_test(|| {
			let mut cell = unsafe {
				RawCell::new_unchecked(Key([0x42; 32]))
			};
			assert_eq!(cell.load(), None);
			cell.store(b"Hello, World!");
			assert_eq!(cell.load(), Some(b"Hello, World!".to_vec()));
			cell.clear();
			assert_eq!(cell.load(), None);
		})
	}

	#[test]
	fn count_reads() {
		run_test(|| {
			let cell = unsafe {
				RawCell::new_unchecked(Key([0x42; 32]))
			};
			assert_eq!(TestEnv::total_reads(), 0);
			cell.load();
			assert_eq!(TestEnv::total_reads(), 1);
			cell.load();
			cell.load();
			assert_eq!(TestEnv::total_reads(), 3);
		})
	}

	#[test]
	fn count_writes() {
		run_test(|| {
			let mut cell = unsafe {
				RawCell::new_unchecked(Key([0x42; 32]))
			};
			assert_eq!(TestEnv::total_writes(), 0);
			cell.store(b"a");
			assert_eq!(TestEnv::total_writes(), 1);
			cell.store(b"b");
			cell.store(b"c");
			assert_eq!(TestEnv::total_writes(), 3);
		})
	}
}
