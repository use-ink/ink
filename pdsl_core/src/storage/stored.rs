//! Convenience utilities for writing smart contracts.

use super::Key;

use std::marker::PhantomData;

/// Pointer to a value stored in the contract storage.
#[derive(Debug, PartialEq, Eq)]
pub struct Stored<T> {
	/// The key associated with the storage.
	key: Key,
	/// Make Rust's type checker happy.
	marker: PhantomData<T>,
}

impl<T> Copy for Stored<T> {}

impl<T> Clone for Stored<T> {
	fn clone(&self) -> Self {
		Stored::from(self.key())
	}
}

impl<T> From<Key> for Stored<T> {
	fn from(key: Key) -> Self {
		Self{key, marker: PhantomData}
	}
}

impl<T> Stored<T> {
	/// Returns the internal key to the associated storage slot.
	pub fn key(&self) -> Key {
		self.key
	}
}

impl<T> Stored<T>
where
	T: parity_codec::Encode
{
	/// Encodes and stores the data to the contract storage.
	pub fn store(self, new_val: &T) {
		self.key.store(&T::encode(&new_val))
	}
}

impl<T> Stored<T>
where
	T: parity_codec::Decode
{
	/// Loads and decodes the data from the contract storage if any.
	///
	/// Returns `None` if the storage slot was empty.
	pub fn load(self) -> Option<T> {
		self.key
			.load()
			.and_then(|bytes| T::decode(&mut &bytes[..]))
	}
}
