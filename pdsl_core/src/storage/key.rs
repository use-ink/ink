use crate::env::{Env, ContractEnv};

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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Key(pub [u8; 32]);

impl Key {
	/// Create a new key from another given key with given offset.
	pub fn with_offset(key: &Key, offset: u32) -> Self {
		let mut offset_key: Self = key.clone();
		utils::bytes_add_u32_inplace(offset_key.as_bytes_mut(), offset);
		offset_key
	}

	/// Returns the byte slice of this key.
	pub(crate) fn as_bytes(&self) -> &[u8] {
		&self.0
	}

	/// Returns the mutable byte slice of this key.
	pub(crate) fn as_bytes_mut(&mut self) -> &mut [u8] {
		&mut self.0
	}

	/// Returns data stored in the storage slot associated with `self`.
	///
	/// # Errors
	///
	/// - Fails if the associated storage slot does not contain data.
	///   This does not count for slots that store empty data.
	pub fn load(&self) -> Option<Vec<u8>> {
		ContractEnv::load(self.as_bytes())
	}

	/// Stores the given data into the storage slot associated with `self`.
	pub fn store(&mut self, value: &[u8]) {
		ContractEnv::store(self.as_bytes(), value)
	}

	/// Clears the storage slot associated with `self`.
	///
	/// Afterswards loading from this slot will fail.
	pub fn clear(&mut self) {
		ContractEnv::clear(self.as_bytes())
	}
}

/// Arithmetic utilities for key manipulation such as integer addition.
///
/// # Note
///
/// This makes it possible to use key arithmetic similar to C's pointer arithmetic.
mod utils {
	/// Converts the given `u32` into a slice of bytes.
	///
	/// The resulting bytes start with the most significant byte
	/// of the given `u32`.
	pub fn u32_to_bytes(val: u32) -> [u8; 4] {
		[
			((val >> 24) & 0xFF) as u8,
			((val >> 16) & 0xFF) as u8,
			((val >>  8) & 0xFF) as u8,
			((val >>  0) & 0xFF) as u8,
		]
	}

	/// Adds the given byte to the given byte slice.
	///
	/// The first byte in the byte slice is interpreted as its
	/// most significant byte.
	pub fn bytes_add_byte_inplace(bytes: &mut [u8], byte: u8) {
		println!("bytes_add_byte_inplace({:?}, {:?})", bytes, byte);
		assert!(bytes.len() > 0);
		match bytes.len() {
			1 => {
				bytes[0] = bytes[0].wrapping_add(byte)
			}
			n => {
				let ls_byte = &mut bytes[n - 1];
				let (res, ovfl) = ls_byte.overflowing_add(byte);
				*ls_byte = res;
				if ovfl {
					bytes_add_byte_inplace(&mut bytes[..(n-1)], 1)
				}
			}
		}
	}

	/// Adds the given `u32` to the given byte slice.
	///
	/// The first byte in the byte slice is interpreted as its
	/// most significant byte.
	pub fn bytes_add_u32_inplace(lhs: &mut [u8], rhs: u32) {
		assert!(lhs.len() >= 4);
		let rhs_bytes = u32_to_bytes(rhs);
		let n = lhs.len();
		for (i, &rhs_byte) in rhs_bytes.iter().rev().enumerate() {
			println!("i = {}", i);
			let lhs_head = &mut lhs[..(n - i)];
			bytes_add_byte_inplace(lhs_head, rhs_byte);
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_u32_to_bytes() {
			assert_eq!(u32_to_bytes(0), [0, 0, 0, 0]);
			assert_eq!(u32_to_bytes(42), [0, 0, 0, 42]);
			assert_eq!(u32_to_bytes(0xFE_DC_BA_98), [0xFE, 0xDC, 0xBA, 0x98]);
			assert_eq!(u32_to_bytes(0xFF_FF_FF_FF), [0xFF, 0xFF, 0xFF, 0xFF]);
		}

		#[test]
		fn test_bytes_add_byte_inplace() {
			fn bytes_add_byte(bytes: &[u8], byte: u8) -> Vec<u8> {
				let mut buffer = bytes.to_vec();
				bytes_add_byte_inplace(&mut buffer, byte);
				buffer
			}

			assert_eq!(bytes_add_byte(&[0x00], 0x00), vec![0x00]);
			assert_eq!(bytes_add_byte(&[0x00], 0x01), vec![0x01]);
			assert_eq!(bytes_add_byte(&[0x00, 0xFF], 0x01), vec![0x01, 0x00]);
			assert_eq!(bytes_add_byte(&[0x00, 0xFF], 0xFF), vec![0x01, 0xFE]);
			assert_eq!(bytes_add_byte(&[0x00, 0xFF, 0xFF], 0xFF), vec![0x01, 0x00, 0xFE]);
		}

		#[test]
		fn test_bytes_add_u32_inplace() {
			fn bytes_add_u32(bytes: &[u8], val: u32) -> Vec<u8> {
				let mut buffer = bytes.to_vec();
				bytes_add_u32_inplace(&mut buffer, val);
				buffer
			}

			assert_eq!(
				bytes_add_u32(&[0x00, 0x00, 0x00, 0x00], 0x0),
				vec![0x00, 0x00, 0x00, 0x00]
			);
			assert_eq!(
				bytes_add_u32(&[0x00, 0x00, 0x00, 0x00], 0x1),
				vec![0x00, 0x00, 0x00, 0x01]
			);
			assert_eq!(
				bytes_add_u32(&[0x00, 0x00, 0x00, 0xFF], 0x1),
				vec![0x00, 0x00, 0x01, 0x00]
			);
			assert_eq!(
				bytes_add_u32(&[0x00, 0x00, 0x00, 0xFF], 0xFF),
				vec![0x00, 0x00, 0x01, 0xFE]
			);
			assert_eq!(
				bytes_add_u32(&[0x00, 0xEF, 0xFF, 0xFF], 0x1),
				vec![0x00, 0xF0, 0x00, 0x00]
			);
			assert_eq!(
				bytes_add_u32(&[0x00, 0xEF, 0xFF, 0xFF], 0xFF),
				vec![0x00, 0xF0, 0x00, 0xFE]
			);
		}
	}
}
