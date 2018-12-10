use crate::{
	storage::{
		Key,
		Stored,
		SyncedRef,
	},
};

use std::collections::HashMap;
use std::cell::UnsafeCell;

/// Synchronized chunk of contract storage.
///
/// The synchronized chunk spans 2^32 storage slots starting from its key.
///
/// # Note
///
/// This is a low-level abstraction layer that synchronizes contract
/// storage and main memory whenever entities are stored or loaded.
///
/// Users of `pwasm` should strictly avoid using it directly!
#[derive(Debug)]
pub struct SyncedChunk<T> {
	/// The key into the synchronized storage.
	key: Key,
	/// The synchronized main memory.
	synced: UnsafeCell<HashMap<u32, T>>,
}

impl<T> From<Key> for SyncedChunk<T> {
	fn from(key: Key) -> Self {
		SyncedChunk{ key, synced: UnsafeCell::new(HashMap::default()) }
	}
}

impl<T> SyncedChunk<T>
where
	T: parity_codec::Codec,
{
	/// Returns a contract storage at the given offset.
	pub(crate) fn storage_at(&self, offset: u32) -> Stored<T> {
		Key::with_offset(&self.key, offset).into()
	}

	/// Loads the given entity at the given offset.
	///
	/// # Note
	///
	/// This does not syncrhonize with main memory.
	fn load_at(&self, offset: u32) -> Option<T> {
		self.storage_at(offset).try_load()
	}

	/// Stores the given value at the given offset.
	///
	/// # Note
	///
	/// This does not syncrhonize with main memory.
	fn store_at(&self, offset: u32, val: &T) {
		self.storage_at(offset).store(val)
	}

	/// Clears the given value at the given offset.
	///
	/// # Note
	///
	/// This does not syncrhonize with main memory.
	fn clear_at(&self, offset: u32) {
		self.storage_at(offset).key().clear()
	}

	/// Returns a mutable reference to the hashmap
	/// that stores the on-memory synchronized elements.
	fn map_mut(&self) -> &mut HashMap<u32, T> {
		let ptr: *mut HashMap<u32, T> = self.synced.get();
		unsafe { &mut *ptr }
	}

	/// Returns a reference to the synchronized element
	/// stored at position `n` from the starting key storage slot.
	///
	/// Returns `None` if there is no element for that slot.
	pub fn get(&self, n: u32) -> Option<&T> {
		if let Some(loaded) = self.load_at(n) {
			use std::collections::hash_map::Entry;
			match self.map_mut().entry(n) {
				Entry::Occupied(mut occupied) => {
					occupied.insert(loaded);
					return Some(occupied.into_mut())
				}
				Entry::Vacant(vacant) => {
					return Some(vacant.insert(loaded))
				}
			}
		}
		None
	}

	/// Returns a mutable reference to the synchronized element
	/// stored at position `n` from the starting key storage slot.
	///
	/// Returns `None` if there is no element for that slot.
	///
	/// # Note
	///
	/// Instead of returning a raw mutable reference it returns a
	/// reference that automatically synchronizes upon manipulation.
	pub fn get_mut(&self, n: u32) -> Option<SyncedRef<T>> {
		if let Some(loaded) = self.load_at(n) {
			use std::collections::hash_map::Entry;
			match self.map_mut().entry(n) {
				Entry::Occupied(mut occupied) => {
					occupied.insert(loaded);
					return Some(
						SyncedRef::new(
							self.storage_at(n),
							occupied.into_mut()
						)
					)
				}
				Entry::Vacant(vacant) => {
					return Some(
						SyncedRef::new(
							self.storage_at(n),
							vacant.insert(loaded)
						)
					)
				}
			}
		}
		None
	}

	/// Inserts an entity into the n-th storage slot starting
	/// from the key slot.
	///
	/// # Note
	///
	/// This will overwrite an already existing element.
	/// The inserted element is going to be synchonized from then on.
	pub fn insert(&mut self, n: u32, val: T) {
		self.store_at(n, &val);
		self.map_mut().insert(n, val);
	}

	/// Removes the entity stored at the n-th storage slot
	/// starting from the key slot.
	///
	/// # Note
	///
	/// Returns the value that was previously stored in that slot
	/// and otherwise `None`.
	pub fn remove(&mut self, n: u32) -> Option<T> {
		if let Some(val) = self.load_at(n) {
			self.clear_at(n);
			self.map_mut().remove(&n);
			return Some(val)
		}
		None
	}
}
