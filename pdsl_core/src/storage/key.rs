use crate::byte_utils;

use parity_codec_derive::{Encode, Decode};

/// Typeless generic key into contract storage.
///
/// # Note
///
/// This is the most low-level method to access contract storage.
///
/// # Unsafe
///
/// - Does not restrict ownership.
/// - Can read and write to any storage location.
/// - Does not synchronize between main memory and contract storage.
/// - Violates Rust's mutability and immutability guarantees.
///
/// Prefer using types found in `collections` or `Synced` type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Encode, Decode)]
pub struct Key(pub [u8; 32]);

impl Key {
	/// Create a new key from another given key with given offset.
	pub fn with_offset(key: Key, offset: u32) -> Self {
		let mut result = key.clone();
		let mut by_array = [0x0; 32];
		(&mut by_array[28..32]).copy_from_slice(&byte_utils::u32_to_bytes4(offset));
		byte_utils::bytes_add_bytes(result.as_bytes_mut(), &by_array);
		result
	}

	/// Create a new key from another given key with given chunk offset.
	///
	/// # Note
	///
	/// A chunk offset is an offset that is a multiple of the chunk size.
	/// The chunk size is 2^32.
	pub fn with_chunk_offset(key: Key, offset: u32) -> Self {
		let mut result = key.clone();
		let mut by_array = [0x0; 32];
		(&mut by_array[24..32]).copy_from_slice(&byte_utils::u64_to_bytes8(
			(1 << 32) * (offset as u64)
		));
		byte_utils::bytes_add_bytes(result.as_bytes_mut(), &by_array);
		result
	}

	/// Returns the byte slice of this key.
	pub fn as_bytes(&self) -> &[u8] {
		&self.0
	}

	/// Returns the mutable byte slice of this key.
	pub fn as_bytes_mut(&mut self) -> &mut [u8] {
		&mut self.0
	}
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
	use super::*;

	use crate::env::{Env, ContractEnv};

	#[test]
	fn store_load_clear() {
		let key = Key([0x42; 32]);
		assert_eq!(ContractEnv::load(key), None);
		ContractEnv::store(key, &[0x5]);
		assert_eq!(ContractEnv::load(key), Some(vec![0x5]));
		ContractEnv::clear(key);
		assert_eq!(ContractEnv::load(key), None);
	}

	#[test]
	fn key_with_offset() {
		let key00 = Key([0x0; 32]);
		let key05 = Key::with_offset(key00, 5);  // -> 5
		let key10 = Key::with_offset(key00, 10); // -> 10         | same as key55
		let key55 = Key::with_offset(key05, 5);  // -> 5 + 5 = 10 | same as key10
		ContractEnv::store(key55, &[42]);
		assert_eq!(ContractEnv::load(key10), Some(vec![42]));
		ContractEnv::store(key10, &[13, 37]);
		assert_eq!(ContractEnv::load(key55), Some(vec![13, 37]));
	}

	#[test]
	fn as_bytes() {
		let mut key = Key([0x42; 32]);
		assert_eq!(key.as_bytes(), &[0x42; 32]);
		assert_eq!(key.as_bytes_mut(), &mut [0x42; 32]);
	}
}
