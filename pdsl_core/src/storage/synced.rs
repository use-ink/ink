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
	/// Synchronizes storage with main memory.
	fn sync(&self) {
		let ptr: *mut Option<T> = self.synced.get() as *mut Option<T>;
		let snc: &mut Option<T> = unsafe { &mut *ptr };
		if snc.is_none() {
			*snc = Some(self.stored.load())
		}
	}

	/// Returns a reference to the synced element.
	pub fn get(&self) -> &T {
		self.sync();
		let ptr: *const Option<T> = self.synced.get() as *const Option<T>;
		unsafe { (&*ptr).as_ref().unwrap() }
	}

	/// Returns a special mutable reference to the synced element.
	pub fn get_mut(&mut self) -> SyncedRef<T> {
		self.sync();
		SyncedRef{
			stored: self.stored.clone(),
			synref: {
				let ptr: *mut Option<T> = self.synced.get() as *mut Option<T>;
				unsafe { (&mut *ptr).as_mut().unwrap() }
			}
		}
	}

	/// Sets the synced element to a new value.
	pub fn set(&mut self, val: T) {
		self.stored.store(&val);
		self.sync();
	}
}

/// A mutable reference to a synced element.
///
/// This will keep the referenced element in sync.
pub struct SyncedRef<'a, T> {
	stored: Stored<T>,
	synref: &'a mut T,
}

impl<'a, T> SyncedRef<'a, T>
where
	T: parity_codec::Encode
{
	/// Creates a new synced reference.
	///
	/// This allows mutable reference access to entities that
	/// are synchronized to the contract storage.
	pub(crate) fn new(stored: Stored<T>, synref: &'a mut T) -> Self {
		Self{ stored, synref }
	}

	/// Returns a read-only reference to the synced entity.
	pub fn get(&self) -> &T {
		&self.synref
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
		f(self.synref);
		self.stored.store(&self.synref);
	}
}
