use crate::{
	storage::{
		Key,
		chunk::{
			TypedChunk,
			error::{
				Result,
				ChunkError,
			}
		},
	},
};

use std::{
	collections::HashMap,
	cell::RefCell
};

/// A chunk of copy cells.
///
/// Provides interpreted and read-optimized access to the associated constract storage slot.
///
/// # Guarantees
///
/// - `Owned`
/// - `Typed`
/// - `Avoid Reads`
///
/// Read more about kinds of guarantees and their effect [here](../index.html#guarantees).
#[derive(Debug, PartialEq, Eq)]
pub struct CopyChunk<T>
where
	T: Copy
{
	/// The underlying chunk of cells.
	chunk: TypedChunk<T>,
	/// The cached elements.
	elems: Cache<T>,
}

#[derive(Debug, PartialEq, Eq)]
struct Cache<T>
where
	T: Copy
{
	/// The synchronized elements.
	elems: RefCell<HashMap<u32, Option<T>>>,
}

impl<T> Default for Cache<T>
where
	T: Copy
{
	fn default() -> Self {
		Self{ elems: RefCell::new(HashMap::new()) }
	}
}

/// A cached entity.
///
/// This is either in sync with the contract storage or out of sync.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Cached<T>
where
	T: Copy
{
	Desync,
	Sync(Option<T>),
}

/// A cache for all synchronized elements.
///
/// # Note
///
/// An element counts as synchronized if its version in the contract
/// storage and the version in the cache are identical.
impl<T> Cache<T>
where
	T: Copy
{
	/// Inserts or updates the element at offset `n`.
	pub fn insert(&self, n: u32, val: Option<T>) {
		self.elems.borrow_mut().insert(n, val);
	}

	/// Returns the element at offset `n`.
	pub fn get(&self, n: u32) -> Cached<T> {
		match self.elems.borrow().get(&n).map(|&ret| ret) {
			Some(opt_elem) => Cached::Sync(opt_elem),
			None => Cached::Desync,
		}
	}
}

impl<T> CopyChunk<T>
where
	T: Copy
{
	/// Creates a new copy cell chunk for the given key and capacity.
	///
	/// # Note
	///
	/// This is unsafe because ..
	/// - .. it does not check if the associated
	///   contract storage does not alias with other accesses.
	/// - .. it does not check if given capacity is non zero.
	pub unsafe fn new_unchecked(key: Key, capacity: u32) -> Self {
		Self{
			chunk: TypedChunk::new_unchecked(key, capacity),
			elems: Cache::default(),
		}
	}

	/// Returns the length of this chunk.
	///
	/// # Note
	///
	/// The returned length is guaranteed to always be greater than zero.
	pub fn capacity(&self) -> u32 {
		self.chunk.capacity()
	}
}

impl<T> CopyChunk<T>
where
	T: Copy + parity_codec::Decode
{
	/// Returns the entity at offset `n` if any.
	///
	/// # Note
	///
	/// This avoid unnecesary contract storage read access if possible.
	///
	/// # Errors
	///
	/// - If `n` is out of bounds.
	pub fn get(&self, n: u32) -> Result<Option<T>> {
		if n >= self.capacity() {
			return Err(ChunkError::access_out_of_bounds(n, self.capacity()))
		}
		if let Cached::Sync(cached) = self.elems.get(n) {
			return Ok(cached)
		}
		self.load(n)
	}

	/// Loads the entity at offset `n` if any.
	///
	/// # Note
	///
	/// This doesn't use optimized read access.
	/// Prefer using `CopyChunk::get` instead of this.
	///
	/// # Errors
	///
	/// - If `n` is out of bounds.
	pub fn load(&self, n: u32) -> Result<Option<T>> {
		// Operation on chunk must come first since it checks for errors.
		let loaded = self.chunk.load(n)?;
		self.elems.insert(n, loaded);
		Ok(loaded)
	}
}

impl<T> CopyChunk<T>
where
	T: Copy + parity_codec::Encode
{
	/// Sets the entity to the given entity.
	///
	/// # Note
	///
	/// This always accesses the contract storage.
	///
	/// # Errors
	///
	/// - If `n` is out of bounds.
	pub fn set(&mut self, n: u32, val: T) -> Result<()> {
		// Operation on chunk must come first since it checks for errors.
		self.chunk.store(n, &val)?;
		self.elems.insert(n, Some(val));
		Ok(())
	}

	/// Removes the entity from the contract storage.
	///
	/// # Errors
	///
	/// - If `n` is out of bounds.
	pub fn clear(&mut self, n: u32) -> Result<()> {
		// Operation on chunk must come first since it checks for errors.
		self.chunk.clear(n)?;
		self.elems.insert(n, None);
		Ok(())
	}
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
	use super::*;

	use crate::env::TestEnv;

	#[test]
	fn simple() {
		const CAPACITY: u32 = 5;

		let mut chunk = unsafe {
			CopyChunk::new_unchecked(Key([0x42; 32]), CAPACITY)
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
			assert!(chunk.set(i, i).is_ok());
			assert_eq!(chunk.get(i), Ok(Some(i)));
			assert_eq!(chunk.load(i), Ok(Some(i)));
		}
		assert_eq!(chunk.capacity(), CAPACITY);

		// Out of bounds storing.
		assert!(chunk.set(CAPACITY, 10).is_err());

		// Clear all elements.
		for i in 0..CAPACITY {
			assert!(chunk.clear(i).is_ok());
			assert_eq!(chunk.get(i), Ok(None));
			assert_eq!(chunk.load(i), Ok(None));
		}
		assert_eq!(chunk.capacity(), CAPACITY);

		// Clear out of bounds.
		assert!(chunk.clear(CAPACITY).is_err());
	}

	#[test]
	fn count_reads_writes() {
		const CAPACITY: u32 = 5;

		let mut chunk = unsafe {
			CopyChunk::new_unchecked(Key([0x42; 32]), CAPACITY)
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
			chunk.set(i, i).unwrap();
			assert_eq!(TestEnv::total_reads(), CAPACITY as u64);
			assert_eq!(TestEnv::total_writes(), i as u64 + 1);
		}
		assert_eq!(TestEnv::total_reads(), CAPACITY as u64);
		assert_eq!(TestEnv::total_writes(), CAPACITY as u64);

		// Loading multiple times from a single cell.
		const LOAD_REPEATS: usize = 3;
		for _ in 0..LOAD_REPEATS {
			chunk.get(0).unwrap();
			assert_eq!(TestEnv::total_reads(), CAPACITY as u64);
			assert_eq!(TestEnv::total_writes(), CAPACITY as u64);
		}

		// Storing multiple times to a single cell.
		const STORE_REPEATS: usize = 3;
		for n in 0..STORE_REPEATS {
			chunk.set(0, 10).unwrap();
			assert_eq!(TestEnv::total_reads(), CAPACITY as u64);
			assert_eq!(TestEnv::total_writes(), CAPACITY as u64 + n as u64 + 1);
		}
		assert_eq!(TestEnv::total_reads(), CAPACITY as u64);
		assert_eq!(TestEnv::total_writes(), CAPACITY as u64 + STORE_REPEATS as u64);
	}
}
