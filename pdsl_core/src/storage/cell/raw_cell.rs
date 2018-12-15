use crate::{
	storage::{
		Key,
		NonCloneMarker,
	},
	env::{Env, ContractEnv},
};

/// A raw cell.
///
/// Provides uninterpreted and unformatted access to the associated contract storage slot.
///
/// # Guarantees
///
/// - `Owned`
///
/// Read more about kinds of guarantees and their effect [here](../index.html#guarantees).
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RawCell {
	/// The key to the associated constract storage slot.
	key: Key,
	/// Marker that prevents this type from being `Copy` or `Clone` by accident.
	non_clone: NonCloneMarker<()>,
}

impl RawCell {
	/// Creates a new raw cell for the given key.
	///
	/// # Note
	///
	/// This is unsafe since it does not check if the associated
	/// contract storage does not alias with other accesses.
	pub unsafe fn new_unchecked(key: Key) -> Self {
		Self{
			key: key,
			non_clone: NonCloneMarker::default()
		}
	}
}

impl RawCell {
	/// Loads the data if any.
	pub fn load(&self) -> Option<Vec<u8>> {
		ContractEnv::load(self.key)
	}

	/// Stores the given data.
	pub fn store(&mut self, bytes: &[u8]) {
		ContractEnv::store(self.key, bytes)
	}

	/// Removes the data from the associated contract storage slot.
	pub fn clear(&mut self) {
		ContractEnv::clear(self.key)
	}
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
	use super::*;

	use crate::env::TestEnv;

	#[test]
	fn simple() {
		let mut cell = unsafe {
			RawCell::new_unchecked(Key([0x42; 32]))
		};
		assert_eq!(cell.load(), None);
		cell.store(b"Hello, World!");
		assert_eq!(cell.load(), Some(b"Hello, World!".to_vec()));
		cell.clear();
		assert_eq!(cell.load(), None);
	}

	#[test]
	fn count_reads() {
		let cell = unsafe {
			RawCell::new_unchecked(Key([0x42; 32]))
		};
		assert_eq!(TestEnv::total_reads(), 0);
		cell.load();
		assert_eq!(TestEnv::total_reads(), 1);
		cell.load();
		cell.load();
		assert_eq!(TestEnv::total_reads(), 3);
	}

	#[test]
	fn count_writes() {
		let mut cell = unsafe {
			RawCell::new_unchecked(Key([0x42; 32]))
		};
		assert_eq!(TestEnv::total_writes(), 0);
		cell.store(b"a");
		assert_eq!(TestEnv::total_writes(), 1);
		cell.store(b"b");
		cell.store(b"c");
		assert_eq!(TestEnv::total_writes(), 3);
	}
}
