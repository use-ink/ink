use crate::{
	storage::{
		Key,
		cell::TypedCell,
	},
};

use std::cell::Cell;

/// A copy cell.
///
/// Provides interpreted and read-optimized access to the associated constract storage slot.
///
/// # Guarantees
///
/// - `Owned`, `Typed`, `Avoid Reads`
///
/// Read more about kinds of guarantees and their effect [here](../index.html#guarantees).
#[derive(Debug, PartialEq, Eq)]
pub struct CopyCell<T>
where
	T: Copy
{
	/// The typed cell.
	cell: TypedCell<T>,
	/// The cached entity.
	///
	/// This allows to avoid unnecesary read accesses.
	elem: Cell<Cached<T>>,
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

impl<T> CopyCell<T>
where
	T: Copy
{
	/// Creates a new copy cell for the given key.
	///
	/// # Note
	///
	/// This is unsafe since it does not check if the associated
	/// contract storage does not alias with other accesses.
	pub unsafe fn new_unchecked(key: Key) -> Self {
		Self{
			cell: TypedCell::new_unchecked(key),
			elem: Cell::new(Cached::Desync)
		}
	}
}

impl<T> CopyCell<T>
where
	T: Copy + parity_codec::Decode
{
	/// Returns the entity if any.
	///
	/// # Note
	///
	/// This avoid unnecesary contract storage read access if possible.
	pub fn get(&self) -> Option<T> {
		if let Cached::Sync(opt_elem) = self.elem.get() {
			return opt_elem
		}
		self.load()
	}

	/// Loads the entity if any.
	///
	/// # Note
	///
	/// This doesn't use optimized read access.
	/// Prefer using `CopyCell::get` instead of this.
	pub fn load(&self) -> Option<T> {
		let elem = self.cell.load();
		self.elem.set(Cached::Sync(elem));
		elem
	}
}

impl<T> CopyCell<T>
where
	T: Copy + parity_codec::Encode
{
	/// Sets the entity to the given entity.
	///
	/// # Note
	///
	/// This always accesses the contract storage.
	pub fn set(&mut self, val: T) {
		self.elem.set(Cached::Sync(Some(val)));
		self.cell.store(&val);
	}

	/// Removes the entity from the contract storage.
	pub fn clear(&mut self) {
		self.elem.set(Cached::Sync(None));
		self.cell.clear();
	}
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
	use super::*;

	use crate::env::TestEnv;

	#[test]
	fn simple() {
		let mut cell: CopyCell<i32> = unsafe {
			CopyCell::new_unchecked(Key([0x42; 32]))
		};
		assert_eq!(cell.get(), None);
		cell.set(5);
		assert_eq!(cell.get(), Some(5));
		cell.clear();
		assert_eq!(cell.get(), None);
	}

	#[test]
	fn count_reads() {
		let cell: CopyCell<i32> = unsafe {
			CopyCell::new_unchecked(Key([0x42; 32]))
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
		let mut cell: CopyCell<i32> = unsafe {
			CopyCell::new_unchecked(Key([0x42; 32]))
		};
		assert_eq!(TestEnv::total_writes(), 0);
		cell.set(1);
		assert_eq!(TestEnv::total_writes(), 1);
		cell.set(2);
		cell.set(3);
		assert_eq!(TestEnv::total_writes(), 3);
	}
}
