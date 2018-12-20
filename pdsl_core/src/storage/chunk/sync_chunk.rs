use crate::{
	storage::{
		Key,
		chunk::{
			TypedChunk,
			TypedChunkCell,
			error::{
				Result,
				ChunkError,
			}
		},
	},
};

use std::{
	collections::{
		HashMap,
		hash_map::Entry,
	},
	cell::RefCell
};

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
	elems: Cache<T>,
}

/// A single cache entry for a copy chunk cell.
type CacheEntry<'a, T> = Entry<'a, u32, Option<T>>;

/// A single cell within a chunk of copy cells.
#[derive(Debug)]
pub struct SyncChunkCell<'a, T> {
	/// The underlying cell within the chunk of cells.
	cell: TypedChunkCell<'a, T>,
	/// The cached entry for the cell.
	elem: CacheEntry<'a, T>,
}

impl<'a, T> SyncChunkCell<'a, T> {
	/// Creates a new cell within a chunk of copy cells.
	///
	/// # Safety
	///
	/// This is unsafe since it doesn't check aliasing of cells
	/// or if the cell and the cache entry are actually associated
	/// with each other.
	pub(self) unsafe fn new_unchecked(
		cell: TypedChunkCell<'a, T>,
		elem: CacheEntry<'a, T>
	) -> Self {
		Self{cell, elem}
	}

	/// Removes the value stored in this cell.
	pub fn clear(self) {
		let mut this = self;
		match this.elem {
			Entry::Occupied(mut occupied) => {
				this.cell.clear();
				occupied.insert(None);
			}
			Entry::Vacant(vacant) => {
				this.cell.clear();
				vacant.insert(None);
			}
		}
	}
}

impl<'a, T> SyncChunkCell<'a, T>
where
	T: parity_codec::Decode
{
	/// Removes the value from the cell and returns the removed value.
	///
	/// # Note
	///
	/// Prefer using `clear` if you are not interested in the return value.
	#[must_use]
	pub fn remove(self) -> Option<T> {
		let mut this = self;
		match this.elem {
			Entry::Occupied(mut occupied) => {
				this.cell.clear();
				occupied.insert(None)
			}
			Entry::Vacant(vacant) => {
				let old = this.cell.load();
				this.cell.clear();
				vacant.insert(None);
				old
			}
		}
	}
}

impl<'a, T> SyncChunkCell<'a, T>
where
	T: parity_codec::Encode
{
	/// Stores the new value into the cell.
	pub fn set(self, val: T) {
		let mut this = self;
		match this.elem {
			Entry::Occupied(mut occupied) => {
				this.cell.store(&val);
				occupied.insert(Some(val));
			}
			Entry::Vacant(vacant) => {
				this.cell.store(&val);
				vacant.insert(Some(val));
			}
		}
	}
}

impl<'a, T> SyncChunkCell<'a, T>
where
	T: parity_codec::Codec
{
	/// Mutates the value of this cell.
	///
	/// Returns an immutable reference to the result if
	/// a mutation happened, otherwise `None` is returned.
	///
	/// # Note
	///
	/// Prefer using `set` if you are not interested in the return value.
	pub fn mutate_with<F>(self, f: F) -> Option<&'a T>
	where
		F: FnOnce(&mut T)
	{
		let mut this = self;
		match this.elem {
			Entry::Occupied(occupied) => {
				if let Some(elem) = occupied.into_mut() {
					f(elem);
					this.cell.store(elem);
					return Some(&*elem)
				}
				None
			}
			Entry::Vacant(vacant) => {
				let mut ret = false;
				let mut elem = this.cell.load();
				if let Some(elem) = &mut elem {
					f(elem);
					this.cell.store(&*elem);
					ret = true;
				}
				let res = (&*vacant.insert(elem)).into();
				if ret {
					return res
				}
				None
			}
		}
	}

	/// Replaces the value of this cell and returns its previous value.
	///
	/// # Note
	///
	/// Prefer using `set` if you are not interested in the return value.
	#[must_use]
	pub fn replace(self, val: T) -> Option<T> {
		let mut this = self;
		match this.elem {
			Entry::Occupied(mut occupied) => {
				this.cell.store(&val);
				occupied.insert(Some(val))
			}
			Entry::Vacant(vacant) => {
				let old = this.cell.load();
				this.cell.store(&val);
				vacant.insert(Some(val));
				old
			}
		}
	}
}

/// Stores the values of synchronized cells.
///
/// # Note
///
/// An element counts as synchronized if its version in the contract
/// storage and the version in the cache are identical.
#[derive(Debug, PartialEq, Eq)]
struct Cache<T> {
	/// The synchronized values of associated cells.
	elems: RefCell<HashMap<u32, Option<T>>>,
}

impl<T> Default for Cache<T> {
	fn default() -> Self {
		Self{ elems: RefCell::new(HashMap::new()) }
	}
}

/// A cached entity.
///
/// This is either in sync with the contract storage or out of sync.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Cached<T> {
	Desync,
	Sync(Option<T>),
}

impl<T> Cache<T> {
	/// Inserts or updates a value associated with the `n`-th cell.
	///
	/// Returns an immutable reference to the new value.
	pub fn upsert(&self, n: u32, val: Option<T>) -> Option<&T> {
		use std::collections::hash_map::{Entry};
		let elems: &mut HashMap<u32, Option<T>> = unsafe {
			&mut *self.elems.as_ptr()
		};
		match elems.entry(n) {
			Entry::Occupied(mut occupied) => {
				occupied.insert(val);
				(&*occupied.into_mut()).into()
			}
			Entry::Vacant(vacant) => {
				(&*vacant.insert(val)).into()
			}
		}
	}

	/// Returns the synchronized value of the `n`-th cell if any.
	pub fn get(&self, n: u32) -> Cached<&T> {
		let elems: &mut HashMap<u32, Option<T>> = unsafe {
			&mut *self.elems.as_ptr()
		};
		match elems.get(&n) {
			Some(opt_elem) => Cached::Sync(opt_elem.into()),
			None => Cached::Desync,
		}
	}

	/// Returns the cache entry for the `n`-th cell.
	pub fn entry(&mut self, n: u32) -> CacheEntry<T> {
		self.elems.get_mut().entry(n)
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
				elems: Cache::default(),
			})
	}
}

impl<T> SyncChunk<T> {
	/// Creates a new mutable cell chunk for the given key and capacity.
	///
	/// # Safety
	///
	/// This is unsafe because ..
	/// - .. it does not check if the associated
	///   contract storage does not alias with other accesses.
	/// - .. it does not check if given capacity is non zero.
	pub unsafe fn new_unchecked(key: Key, len: u32) -> Self {
		Self{
			chunk: TypedChunk::new_unchecked(key, len),
			elems: Cache::default(),
		}
	}

	/// Returns an accessor to the `n`-th cell.
	pub(crate) fn cell_at(&mut self, n: u32) -> Result<SyncChunkCell<T>> {
		Ok(unsafe {
			SyncChunkCell::new_unchecked(
				self.chunk.cell_at(n)?,
				self.elems.entry(n)
			)
		})
	}

	/// Returns the capacity of this chunk.
	pub fn capacity(&self) -> u32 {
		self.chunk.capacity()
	}

	/// Clear the `n`-th cell.
	///
	/// # Errors
	///
	/// If `n` is out of bounds.
	pub fn clear(&mut self, n: u32) -> Result<()> {
		self.cell_at(n)
			.map(|cell| cell.clear())
	}
}

impl<T> SyncChunk<T>
where
	T: parity_codec::Decode
{
	/// Returns the value of the `n`-th cell if any.
	///
	/// # Errors
	///
	/// If `n` is out of bounds.
	pub fn get(&self, n: u32) -> Result<Option<&T>> {
		if n >= self.capacity() {
			return Err(ChunkError::access_out_of_bounds(n, self.capacity()))
		}
		if let Cached::Sync(cached) = self.elems.get(n) {
			return Ok(cached)
		}
		self.load(n)
	}

	/// Returns the value of the `n`-th cell if any.
	///
	/// # Note
	///
	/// Prefer using [`get`](struct.SyncChunk.html#method.get)
	/// to avoid unnecesary contract storage accesses.
	///
	/// # Errors
	///
	/// If `n` is out of bounds.
	fn load(&self, n: u32) -> Result<Option<&T>> {
		Ok(
			self.elems.upsert(
				n,
				self.chunk.load(n)?
			)
		)
	}

	/// Clears the `n`-th cell and returns its previous value if any.
	///
	/// # Note
	///
	/// Use [`clear`](struct.SyncChunk.html#method.clear) instead
	/// if you are not interested in the old return value.
	///
	/// # Errors
	///
	/// If `n` is out of bounds.
	#[must_use]
	pub fn remove(&mut self, n: u32) -> Result<Option<T>> {
		self.cell_at(n)
			.map(|cell| cell.remove())
	}
}

impl<T> SyncChunk<T>
where
	T: parity_codec::Encode
{
	/// Sets the value of the `n`-th cell.
	///
	/// # Errors
	///
	/// If `n` is out of bounds.
	pub fn set(&mut self, n: u32, val: T) -> Result<()> {
		self.cell_at(n)
			.map(|cell| cell.set(val))
	}
}

impl<T> SyncChunk<T>
where
	T: parity_codec::Codec
{
	/// Sets the value of the `n`-th cell and returns its old value if any.
	///
	/// # Note
	///
	/// Use [`set`](struct.SyncChunk.html#method.set) instead
	/// if you are not interested in the old return value.
	///
	/// # Errors
	///
	/// If `n` is out of bounds.
	#[must_use]
	pub fn replace(&mut self, n: u32, val: T) -> Result<Option<T>> {
		self.cell_at(n)
			.map(|cell| cell.replace(val))
	}

	/// Mutates the value of the `n`-th cell if any.
	///
	/// Returns an immutable reference to the result if
	/// a mutation happened, otherwise `None` is returned.
	///
	/// # Errors
	///
	/// If `n` is out of bounds.
	pub fn mutate_with<F>(&mut self, n: u32, f: F) -> Result<Option<&T>>
	where
		F: FnOnce(&mut T)
	{
		self.cell_at(n)
			.map(|cell| cell.mutate_with(f))
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
			SyncChunk::new_unchecked(Key([0x42; 32]), CAPACITY)
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
			assert_eq!(chunk.get(i), Ok(Some(&i)));
			assert_eq!(chunk.load(i), Ok(Some(&i)));
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
			SyncChunk::new_unchecked(Key([0x42; 32]), CAPACITY)
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

	#[test]
	fn replace() {
		const CAPACITY: u32 = 5;

		let mut chunk = unsafe {
			SyncChunk::new_unchecked(Key([0x42; 32]), CAPACITY)
		};

		// Out of bounds replacement
		assert!(chunk.replace(CAPACITY, 5).is_err());

		// Replace some with none.
		assert_eq!(chunk.replace(0, 42), Ok(None));
		// Again will yield previous result.
		assert_eq!(chunk.replace(0, 42), Ok(Some(42)));

		// After clearing it will be none again.
		assert!(chunk.clear(0).is_ok());
		assert_eq!(chunk.replace(0, 42), Ok(None));
	}

	#[test]
	fn remove() {
		const CAPACITY: u32 = 5;

		let mut chunk = unsafe {
			SyncChunk::new_unchecked(Key([0x42; 32]), CAPACITY)
		};

		// Out of bounds replacement
		assert!(chunk.remove(CAPACITY).is_err());

		// Remove at none.
		assert_eq!(chunk.remove(0), Ok(None));
		// Again will yield none again.
		assert_eq!(chunk.remove(0), Ok(None));
		// Also get will return none.
		assert_eq!(chunk.get(0), Ok(None));

		// After inserting it will yield the inserted value.
		assert!(chunk.set(0, 1337).is_ok());
		// Before remove returns the inserted value.
		assert_eq!(chunk.get(0), Ok(Some(&1337)));
		// Remove yields the removed value.
		assert_eq!(chunk.remove(0), Ok(Some(1337)));
		// After remove returns none again.
		assert_eq!(chunk.get(0), Ok(None));
	}
}
