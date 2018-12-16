use crate::{
	storage::{
		Key,
		NonCloneMarker,
		chunk::{
			RawChunk,
			RawChunkCell,
			error::{
				Result,
			}
		},
	},
};

/// A chunk of typed cells.
///
/// Provides interpreted access with offset to the associated contract storage slot.
///
/// # Guarantees
///
/// - `Owned`
/// - `Typed`
///
/// Read more about kinds of guarantees and their effect [here](../index.html#guarantees).
#[derive(Debug, PartialEq, Eq)]
pub struct TypedChunk<T> {
	/// The underlying chunk of cells.
	chunk: RawChunk,
	/// Marker that prevents this type from being `Copy` or `Clone` by accident.
	non_clone: NonCloneMarker<T>,
}

/// A single cell within a chunk of typed cells.
#[derive(Debug, PartialEq, Eq)]
pub struct TypedChunkCell<'a, T> {
	/// The underlying cell within the chunk of cells.
	cell: RawChunkCell<'a>,
	/// Marker that prevents this type from being `Copy` or `Clone` by accident.
	non_clone: NonCloneMarker<T>,
}

impl<'a, T> TypedChunkCell<'a, T> {
	/// Creates a new raw chunk cell from the given key.
	///
	/// # Safety
	///
	/// This is unsafe since it doesn't check aliasing of cells.
	pub(self) unsafe fn new_unchecked(cell: RawChunkCell<'a>) -> Self {
		Self{
			cell,
			non_clone: NonCloneMarker::default()
		}
	}

	/// Removes the value stored in this cell.
	pub fn clear(&mut self) {
		self.cell.clear()
	}
}

impl<'a, T> TypedChunkCell<'a, T>
where
	T: parity_codec::Decode
{
	/// Loads the value stored in the cell if any.
	///
	/// # Panics
	///
	/// If decoding of the loaded bytes fails.
	pub fn load(&self) -> Option<T> {
		self
			.cell
			.load()
			.map(|loaded| {
				T::decode(&mut &loaded[..])
					// Maybe we should return an error instead of panicking.
					.expect(
						"[pdsl_core::TypedChunkCell::load] Error: \
						 failed upon decoding"
					)
			})
	}
}

impl<'a, T> TypedChunkCell<'a, T>
where
	T: parity_codec::Encode
{
	/// Stores the value into the cell.
	pub fn store(&mut self, val: &T) {
		self.cell.store(&T::encode(val))
	}
}

impl<T> TypedChunk<T> {
	/// Creates a new typed cell chunk for the given key and length.
	///
	/// # Note
	///
	/// This is unsafe because ..
	/// - .. it does not check if the associated
	///   contract storage does not alias with other accesses.
	/// - .. it does not check if given length is non zero.
	pub unsafe fn new_unchecked(key: Key, capacity: u32) -> Self {
		Self{
			chunk: RawChunk::new_unchecked(key, capacity),
			non_clone: NonCloneMarker::default(),
		}
	}

	/// Returns the capacity of this chunk.
	pub fn capacity(&self) -> u32 {
		self.chunk.capacity()
	}

	/// Returns an accessor to the `n`-th cell.
	pub(crate) fn cell_at(&mut self, n: u32) -> Result<TypedChunkCell<T>> {
		self
			.chunk
			.cell_at(n)
			.map(|raw_cell| unsafe {
				TypedChunkCell::new_unchecked(raw_cell)
			})
	}

	/// Removes the value stored in the `n`-th cell.
	pub fn clear(&mut self, n: u32) -> Result<()> {
		self.chunk.clear(n)
	}
}

impl<T> TypedChunk<T>
where
	T: parity_codec::Decode
{
	/// Loads the value stored in the `n`-th cell if any.
	///
	/// # Panics
	///
	/// If decoding of the loaded bytes fails.
	pub fn load(&self, n: u32) -> Result<Option<T>> {
		self
			.chunk
			.load(n)
			.map(|opt_loaded| {
				opt_loaded.map(|loaded| {
					T::decode(&mut &loaded[..])
						// Maybe we should return an error instead of panicking.
						.expect(
							"[pdsl_core::TypedChunk::load] Error: \
							failed upon decoding"
						)
				})
			})
	}
}

impl<T> TypedChunk<T>
where
	T: parity_codec::Encode
{
	/// Stores the value into the `n`-th cell.
	pub fn store(&mut self, n: u32, val: &T) -> Result<()> {
		self.cell_at(n).map(|mut cell| cell.store(val))
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
			TypedChunk::new_unchecked(Key([0x42; 32]), CAPACITY)
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
			assert!(chunk.store(i, &i).is_ok());
			assert_eq!(chunk.load(i), Ok(Some(i)));
		}
		assert_eq!(chunk.capacity(), CAPACITY);

		// Out of bounds storing.
		assert!(chunk.store(CAPACITY, &10).is_err());

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

		let mut chunk = unsafe {
			TypedChunk::new_unchecked(Key([0x42; 32]), CAPACITY)
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
			chunk.store(i, &i).unwrap();
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
			chunk.store(0, &10).unwrap();
			assert_eq!(TestEnv::total_reads(), CAPACITY as u64 + LOAD_REPEATS as u64);
			assert_eq!(TestEnv::total_writes(), CAPACITY as u64 + n as u64 + 1);
		}
		assert_eq!(TestEnv::total_reads(), CAPACITY as u64 + LOAD_REPEATS as u64);
		assert_eq!(TestEnv::total_writes(), CAPACITY as u64 + STORE_REPEATS as u64);
	}
}
