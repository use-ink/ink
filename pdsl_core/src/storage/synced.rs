use crate::{
	storage::{
		Key,
		Stored,
	},
};
use std::cell::UnsafeCell;

/// Synchronized access to the storage.
///
/// # Note
///
/// Synchronizes storage and main memory during contract execution.
/// This is required for collections to behave as if they own their elements.
#[derive(Debug)]
pub struct Synced<T> {
	/// Storage access.
	stored: Stored<T>,
	/// Synced element in main memory.
	///
	/// The `Option<T>` allows for lazy loading.
	synced: UnsafeCell<Option<T>>,
}

impl<T> From<Key> for Synced<T> {
	fn from(key: Key) -> Self {
		Self{
			stored: Stored::from(key),
			synced: UnsafeCell::new(None),
		}
	}
}

impl<T> Synced<T>
where
	T: parity_codec::Codec
{
	/// Returns a mutable reference to the synchronized element.
	///
	/// # Note
	///
	/// This function is inherently unsafe to call since it allows
	/// to mutate state from within a method marked as non-mutating (`&self`).
	fn synced_mut(&self) -> &mut Option<T> {
		let ptr: *mut Option<T> = self.synced.get();
		unsafe { &mut *ptr }
	}

	/// Synchronizes storage with main memory.
	fn sync(&self) {
		*self.synced_mut() = self.stored.try_load();
	}

	/// Returns a reference to the synced element.
	pub fn get(&self) -> Option<&T> {
		self.sync();
		match self.synced_mut() {
			Some(mutref) => Some(&*mutref),
			None => None
		}
	}

	/// Returns a special mutable reference to the synced element.
	pub fn get_mut(&mut self) -> Option<SyncedMut<T>> {
		self.sync();
		match self.synced_mut() {
			Some(mutref) => {
				Some(SyncedMut::from_raw_parts(self.stored, mutref))
			},
			None => None
		}
	}

	/// Sets the synced element to a new value.
	pub fn set(&mut self, val: T) {
		self.stored.store(&val);
		*self.synced_mut() = Some(val);
	}
}

/// A mutable reference to a synced element.
///
/// This will keep the referenced element in sync.
#[derive(Debug)]
pub struct SyncedMut<'a, T> {
	stored: Stored<T>,
	mutref: &'a mut T,
}

impl<'a, T> SyncedMut<'a, T>
where
	T: parity_codec::Encode
{
	/// Creates a new synced reference.
	///
	/// This allows mutable reference access to entities that
	/// are synchronized to the contract storage.
	pub(crate) fn from_raw_parts(stored: Stored<T>, mutref: &'a mut T) -> Self {
		Self{ stored, mutref }
	}

	/// Returns a read-only reference to the synced entity.
	pub fn get(&self) -> &T {
		&self.mutref
	}

	/// Mutates the synced entity inplace with the given closure.
	///
	/// # Note
	///
	/// Synchronizes state with the contract storage immediately after.
	pub fn mutate_with<F>(&mut self, f: F)
	where
		F: FnOnce(&mut T)
	{
		f(self.mutref);
		self.stored.store(&self.mutref);
	}
}
