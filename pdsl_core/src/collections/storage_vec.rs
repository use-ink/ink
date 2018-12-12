use crate::storage::{
	Key,
	Synced,
	SyncedRef,
	SyncedChunk,
};

use std::marker::PhantomData;

/// A storage vector capable of storing elements in the storage
/// in contiguous hashes.
///
/// # Note
///
/// - Due to the architecture of the storage this storage vector is
/// very different from an actual vector such as Rust's `Vec`.
/// Even though its hashes with which it stores elements in the storage
/// are contiguous doesn't mean the elements are stored in a single
/// dense block of memory.
///
/// - This can be used as a building block for other storage data structures.
#[derive(Debug)]
pub struct StorageVec<T> {
	/// The length of this storage vec.
	len: Synced<u32>,
	/// Synced chunk of elements.
	synced: SyncedChunk<T>,
	/// Marker to make Rust's type system happy.
	marker: PhantomData<T>,
}

impl<T> From<Key> for StorageVec<T> {
	fn from(key: Key) -> Self {
		StorageVec{
			len: Synced::from(key),
			synced: SyncedChunk::from(
				Key::with_offset(key, 1)
			),
			marker: PhantomData,
		}
	}
}

impl<T> StorageVec<T> {
	/// Returns the number of elements in the vector.
	pub fn len(&self) -> u32 {
		*self.len.get()
	}

	/// Returns `true` if the vector contains no elements.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

impl<T> StorageVec<T>
where
	T: parity_codec::Codec
{
	/// Returns the n-th elements of this storage vec.
	///
	/// Returns `None` if given `n` is out of bounds.
	pub fn get(&self, n: u32) -> Option<&T> {
		if n >= self.len() {
			return None
		}
		// Some(self.storage_at(n).get())
		Some(self.synced.get(n).expect("TODO"))
	}

	/// Returns a mutable reference to the n-th element of this storage vec.
	///
	/// Returns `None` if given `n` is out of bounds.
	pub fn get_mut(&mut self, n: u32) -> Option<SyncedRef<T>> {
		if n >= self.len() {
			return None
		}
		Some(self.get_mut(n).expect("TODO"))
	}

	/// Appends an element to the back of a collection.
	pub fn push(&mut self, val: T) {
		if self.len() == u32::max_value() {
			panic!(
				"[pdsl_core::StorageVec::push] \
				 Error: cannot push more elements than `u32::MAX`"
			)
		}
		let last_index = self.len();
		self.len.set(last_index + 1);
		self.synced.insert(last_index, val)
	}

	/// Removes the last element from a vector and returns it, or `None` if it is empty.
	pub fn pop(&mut self) -> Option<T> {
		if self.len() == 0 {
			return None
		}
		let last_index = self.len() - 1;
		Some(self.synced.remove(last_index).expect("TODO"))
	}

	/// Removes an element from the vector and returns it.
	/// 
	/// The removed element is replaced by the last element of the vector.
	/// This does not preserve ordering, but is O(1).
	/// 
	/// Returns `None` if empty or if index is out of bounds.
	pub fn swap_remove(&mut self, index: u32) -> Option<T> {
		if index >= self.len() {
			return None
		}
		if self.len() <= 1 {
			return self.pop()
		}
		let ret = self
			.synced.remove(index)
			.expect(
				"[pdsl_core::StorageVec::swap_remove] \
				 Error: expected element at given index"
			);
		let last = self
			.pop()
			.expect(
				"[pdsl_core::StorageVec::swap_remove] \
				 Error: expected element for pop()"
			);
		self.synced.insert(index, last);
		Some(ret)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	use crate::env::{Env, TestEnv};
	use parity_codec::{Encode};

	#[test]
	fn new() {
		let k0 = Key([0x0; 32]);
		{
			TestEnv::store(k0.as_bytes(), &u32::encode(&0));
			assert_eq!(StorageVec::<i32>::from(k0).len(), 0);
		}
		{
			TestEnv::reset(); // Not necesarily required.
			TestEnv::store(k0.as_bytes(), &u32::encode(&42));
			assert_eq!(StorageVec::<i32>::from(k0).len(), 42);
		}
	}
}
