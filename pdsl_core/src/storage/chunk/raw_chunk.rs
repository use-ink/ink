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

/// A chunk of raw cells.
///
/// Provides uninterpreted and unformatted access with offset
/// to the associated contract storage slot.
///
/// # Guarantees
///
/// - `Owned`
///
/// Read more about kinds of guarantees and their effect [here](../index.html#guarantees).
#[derive(Debug, PartialEq, Eq)]
pub struct RawChunk {
	/// The key to the associated constract storage slot.
	key: Key,
	/// Marker that prevents this type from being `Copy` or `Clone` by accident.
	non_clone: NonCloneMarker<()>,
}

/// A single cell within a chunk of raw cells.
#[derive(Debug, PartialEq, Eq)]
pub struct RawChunkCell<'a> {
	/// The key to the corresponding cell within the raw chunk.
	key: Key,
	/// Marker that prevents this type from being `Copy` or `Clone` by accident.
	non_clone: NonCloneMarker<&'a mut ()>,
}

impl RawChunkCell<'_> {
	/// Creates a new raw chunk cell from the given key.
	///
	/// # Safety
	///
	/// This is unsafe since it doesn't check aliasing of cells.
	pub(self) unsafe fn new_unchecked(key: Key) -> Self {
		Self{
			key,
			non_clone: NonCloneMarker::default()
		}
	}

	/// Store the bytes into the cell.
	pub fn store(&mut self, bytes: &[u8]) {
		unsafe { ContractEnv::store(self.key, bytes) }
	}

	/// Remove the bytes stored in the cell.
	pub fn clear(&mut self) {
		unsafe { ContractEnv::clear(self.key) }
	}
}

impl parity_codec::Encode for RawChunk {
	fn encode_to<W: parity_codec::Output>(&self, dest: &mut W) {
		self.key.encode_to(dest)
	}
}

impl parity_codec::Decode for RawChunk {
	fn decode<I: parity_codec::Input>(input: &mut I) -> Option<Self> {
		Key::decode(input)
			.map(|key| unsafe {
				Self::new_unchecked(key)
			})
	}
}

impl RawChunk {
	/// Creates a new raw cell chunk for the given key and capacity.
	///
	/// # Safety
	///
	/// This is unsafe because ...
	/// - ... it does not check if the associated
	///   contract storage does not alias with other accesses.
	/// - ... it does not check if given capacity is non zero.
	unsafe fn new_unchecked(key: Key) -> Self {
		Self{
			key,
			non_clone: Default::default(),
		}
	}

	/// Allocates a new raw cell chunk using the given storage allocator.
	///
	/// # Safety
	///
	/// This is unsafe because it does not check if the associated storage
	/// does not alias with storage allocated by other storage allocators.
	pub unsafe fn new_using_alloc<A>(alloc: &mut A) -> Self
	where
		A: Allocator
	{
		Self::new_unchecked(alloc.alloc(u32::max_value()))
	}

	/// Returns the unterlying key to the cells.
	///
	/// # Note
	///
	/// This is a low-level utility getter and should
	/// normally not be required by users.
	pub fn cells_key(&self) -> Key {
		self.key
	}

	/// Returns a key for the `n`-th cell if within bounds.
	///
	/// # Error
	///
	/// Returns an error if `n` is not within bounds.
	fn offset_key(&self, n: u32) -> Key {
		self.key + n
	}

	/// Returns an accessor to the `n`-th cell.
	pub(crate) fn cell_at(&mut self, n: u32) -> RawChunkCell {
		unsafe {
			RawChunkCell::new_unchecked(self.offset_key(n))
		}
	}

	/// Loads the bytes stored in the `n`-th cell.
	pub fn load(&self, n: u32) -> Option<Vec<u8>> {
		unsafe { ContractEnv::load(self.key + n) }
	}

	/// Stores the given bytes into the `n`-th cell.
	pub fn store(&mut self, n: u32, bytes: &[u8]) {
		self.cell_at(n).store(bytes)
	}

	/// Removes the bytes stored in the `n`-th cell.
	pub fn clear(&mut self, n: u32) {
		self.cell_at(n).clear()
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
			const TEST_LEN: u32 = 5;
			const WORD_SIZE: usize = 4;

			let mut chunk = unsafe {
				RawChunk::new_unchecked(Key([0x42; 32]))
			};

			// Invariants after initialization
			for i in 0..TEST_LEN {
				assert_eq!(chunk.load(i), None);
			}

			// Store some elements
			for i in 0..TEST_LEN {
				chunk.store(i, &[i as u8; WORD_SIZE]);
				assert_eq!(chunk.load(i), Some(vec![i as u8; WORD_SIZE]));
			}

			// Clear all elements.
			for i in 0..TEST_LEN {
				chunk.clear(i);
				assert_eq!(chunk.load(i), None);
			}
		})
	}

	#[test]
	fn count_reads_writes() {
		run_test(|| {
			const TEST_LEN: u32 = 5;
			const WORD_SIZE: usize = 4;

			let mut chunk = unsafe {
				RawChunk::new_unchecked(Key([0x42; 32]))
			};

			// Reads and writes after init.
			assert_eq!(TestEnv::total_reads(), 0);
			assert_eq!(TestEnv::total_writes(), 0);

			// Loading from all cells.
			for i in 0..TEST_LEN {
				chunk.load(i);
				assert_eq!(TestEnv::total_reads(), i as u64 + 1);
				assert_eq!(TestEnv::total_writes(), 0);
			}
			assert_eq!(TestEnv::total_reads(), TEST_LEN as u64);
			assert_eq!(TestEnv::total_writes(), 0);

			// Writing to all cells.
			for i in 0..TEST_LEN {
				chunk.store(i, &[i as u8; WORD_SIZE]);
				assert_eq!(TestEnv::total_reads(), TEST_LEN as u64);
				assert_eq!(TestEnv::total_writes(), i as u64 + 1);
			}
			assert_eq!(TestEnv::total_reads(), TEST_LEN as u64);
			assert_eq!(TestEnv::total_writes(), TEST_LEN as u64);

			// Loading multiple times from a single cell.
			const LOAD_REPEATS: usize = 3;
			for n in 0..LOAD_REPEATS {
				chunk.load(0);
				assert_eq!(TestEnv::total_reads(), TEST_LEN as u64 + n as u64 + 1);
				assert_eq!(TestEnv::total_writes(), TEST_LEN as u64);
			}
			assert_eq!(TestEnv::total_reads(), TEST_LEN as u64 + LOAD_REPEATS as u64);
			assert_eq!(TestEnv::total_writes(), TEST_LEN as u64);

			// Storing multiple times to a single cell.
			const STORE_REPEATS: usize = 3;
			for n in 0..STORE_REPEATS {
				chunk.store(0, b"test");
				assert_eq!(TestEnv::total_reads(), TEST_LEN as u64 + LOAD_REPEATS as u64);
				assert_eq!(TestEnv::total_writes(), TEST_LEN as u64 + n as u64 + 1);
			}
			assert_eq!(TestEnv::total_reads(), TEST_LEN as u64 + LOAD_REPEATS as u64);
			assert_eq!(TestEnv::total_writes(), TEST_LEN as u64 + STORE_REPEATS as u64);
		})
	}
}
