use crate::{
	storage::{
		Key,
		NonCloneMarker,
		chunk::{
			error::{ChunkError, Result},
		},
	},
	env::{Env, ContractEnv},
};
use std::num::NonZeroU32;

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
	/// The number of associated contract storage slots.
	capacity: NonZeroU32,
	/// Marker that prevents this type from being `Copy` or `Clone` by accident.
	non_clone: NonCloneMarker<()>,
}

impl RawChunk {
	/// Creates a new raw cell chunk for the given key and capacity.
	///
	/// # Note
	///
	/// This is unsafe because ...
	/// - ... it does not check if the associated
	///   contract storage does not alias with other accesses.
	/// - ... it does not check if given capacity is non zero.
	pub unsafe fn new_unchecked(key: Key, capacity: u32) -> Self {
		Self{
			key,
			capacity: NonZeroU32::new_unchecked(capacity),
			non_clone: NonCloneMarker::default(),
		}
	}

	/// Returns `Ok` if `n` is within bounds for `self`.
	///
	/// # Error
	///
	/// Returns an error if `n` is not within bounds.
	fn offset_key(&self, n: u32) -> Result<Key> {
		if n >= self.capacity() {
			return Err(ChunkError::access_out_of_bounds(n, self.capacity()))
		}
		Ok(Key::with_offset(self.key, n))
	}

	/// Returns the capacity of this chunk.
	///
	/// # Note
	///
	/// The returned length is guaranteed to always be greater than zero.
	pub fn capacity(&self) -> u32 {
		self.capacity.get()
	}

	/// Loads the data at offset `n` if any.
	pub fn load(&self, n: u32) -> Result<Option<Vec<u8>>> {
		self.offset_key(n).map(|key| {
			ContractEnv::load(key)
		})
	}

	/// Stores the given data at offset `n`.
	pub fn store(&mut self, n: u32, bytes: &[u8]) -> Result<()> {
		self.offset_key(n).map(|key| {
			ContractEnv::store(key, bytes)
		})
	}

	/// Removes the data at offset `n` from the associated contract storage slot.
	pub fn clear(&mut self, n: u32) -> Result<()> {
		self.offset_key(n).map(|key| {
			ContractEnv::clear(key)
		})
	}
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
	use super::*;

	use crate::env::TestEnv;

	#[test]
	fn simple() {
		const CAPACITY: u32 = 5;
		const WORD_SIZE: usize = 4;

		let mut chunk = unsafe {
			RawChunk::new_unchecked(Key([0x42; 32]), CAPACITY)
		};

		// Invariants after initialization
		assert_eq!(chunk.capacity(), CAPACITY);
		for i in 0..CAPACITY {
			assert_eq!(chunk.load(i), Ok(None));
		}
		// Out of bounds load.
		assert!(chunk.load(CAPACITY).is_err());

		// Store some elements
		for i in 0..CAPACITY {
			assert!(chunk.store(i, &[i as u8; WORD_SIZE]).is_ok());
			assert_eq!(chunk.load(i), Ok(Some(vec![i as u8; WORD_SIZE])));
		}
		assert_eq!(chunk.capacity(), CAPACITY);

		// Out of bounds storing.
		assert!(chunk.store(CAPACITY, &[10; WORD_SIZE]).is_err());

		// Clear all elements.
		for i in 0..CAPACITY {
			assert!(chunk.clear(i).is_ok());
			assert_eq!(chunk.load(i), Ok(None));
		}
		assert_eq!(chunk.capacity(), CAPACITY);

		// Clear out of bounds.
		assert!(chunk.clear(CAPACITY).is_err());
	}

	#[test]
	fn count_reads_writes() {
		const CAPACITY: u32 = 5;
		const WORD_SIZE: usize = 4;

		let mut chunk = unsafe {
			RawChunk::new_unchecked(Key([0x42; 32]), CAPACITY)
		};

		// Reads and writes after init.
		assert_eq!(TestEnv::total_reads(), 0);
		assert_eq!(TestEnv::total_writes(), 0);

		// Loading from all cells.
		for i in 0..CAPACITY {
			chunk.load(i).unwrap();
			assert_eq!(TestEnv::total_reads(), i as u64 + 1);
			assert_eq!(TestEnv::total_writes(), 0);
		}
		assert_eq!(TestEnv::total_reads(), CAPACITY as u64);
		assert_eq!(TestEnv::total_writes(), 0);

		// Writing to all cells.
		for i in 0..CAPACITY {
			chunk.store(i, &[i as u8; WORD_SIZE]).unwrap();
			assert_eq!(TestEnv::total_reads(), CAPACITY as u64);
			assert_eq!(TestEnv::total_writes(), i as u64 + 1);
		}
		assert_eq!(TestEnv::total_reads(), CAPACITY as u64);
		assert_eq!(TestEnv::total_writes(), CAPACITY as u64);

		// Loading multiple times from a single cell.
		const LOAD_REPEATS: usize = 3;
		for n in 0..LOAD_REPEATS {
			chunk.load(0).unwrap();
			assert_eq!(TestEnv::total_reads(), CAPACITY as u64 + n as u64 + 1);
			assert_eq!(TestEnv::total_writes(), CAPACITY as u64);
		}
		assert_eq!(TestEnv::total_reads(), CAPACITY as u64 + LOAD_REPEATS as u64);
		assert_eq!(TestEnv::total_writes(), CAPACITY as u64);

		// Storing multiple times to a single cell.
		const STORE_REPEATS: usize = 3;
		for n in 0..STORE_REPEATS {
			chunk.store(0, b"test").unwrap();
			assert_eq!(TestEnv::total_reads(), CAPACITY as u64 + LOAD_REPEATS as u64);
			assert_eq!(TestEnv::total_writes(), CAPACITY as u64 + n as u64 + 1);
		}
		assert_eq!(TestEnv::total_reads(), CAPACITY as u64 + LOAD_REPEATS as u64);
		assert_eq!(TestEnv::total_writes(), CAPACITY as u64 + STORE_REPEATS as u64);
	}
}
