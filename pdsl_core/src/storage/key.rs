// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

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
	/// Returns the byte slice of this key.
	pub fn as_bytes(&self) -> &[u8] {
		&self.0
	}

	/// Returns the mutable byte slice of this key.
	pub fn as_bytes_mut(&mut self) -> &mut [u8] {
		&mut self.0
	}
}

impl core::ops::Sub for Key {
	type Output = KeyDiff;

	fn sub(self, rhs: Self) -> KeyDiff {
		let mut lhs = self;
		let mut rhs = rhs;
		byte_utils::negate_bytes(rhs.as_bytes_mut());
		byte_utils::bytes_add_bytes(lhs.as_bytes_mut(), rhs.as_bytes());
		KeyDiff(lhs.0)
	}
}

/// The difference between two keys.
///
/// This is the result of substracting one key from another.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyDiff([u8; 32]);

impl KeyDiff {
	/// Returns the byte slice of this key difference.
	fn as_bytes(&self) -> &[u8] {
		&self.0
	}

	/// Tries to convert the key difference to a `u32` if possible.
	///
	/// Returns `None` if the resulting value is out of bounds.
	pub fn try_to_u32(&self) -> Option<u32> {
		const KEY_BYTES: usize = 32;
		const U32_BYTES: usize = core::mem::size_of::<u32>();
		if self.as_bytes().into_iter().take(KEY_BYTES - U32_BYTES).any(|&byte| byte != 0x0) {
			return None
		}
		let value = byte_utils::bytes4_to_u32(
			byte_utils::slice4_as_array4(&self.as_bytes()[(KEY_BYTES - U32_BYTES)..KEY_BYTES])
				.unwrap()
		);
		Some(value)
	}

	/// Tries to convert the key difference to a `u64` if possible.
	///
	/// Returns `None` if the resulting value is out of bounds.
	pub fn try_to_u64(&self) -> Option<u64> {
		const KEY_BYTES: usize = 32;
		const U64_BYTES: usize = core::mem::size_of::<u64>();
		if self.as_bytes().into_iter().take(KEY_BYTES - U64_BYTES).any(|&byte| byte != 0x0) {
			return None
		}
		let value = byte_utils::bytes8_to_u64(
			byte_utils::slice8_as_array8(&self.as_bytes()[(KEY_BYTES - U64_BYTES)..KEY_BYTES])
				.unwrap()
		);
		Some(value)
	}
}

impl core::ops::Add<u32> for Key {
	type Output = Key;

	fn add(self, rhs: u32) -> Self::Output {
		let mut result = self;
		let ovfl = byte_utils::bytes_add_bytes(
			result.as_bytes_mut(),
			&byte_utils::u32_to_bytes4(rhs)
		);
		assert!(!ovfl, "overflows should not occure for 256-bit keys");
		result
	}
}

impl core::ops::AddAssign<u32> for Key {
	fn add_assign(&mut self, rhs: u32) {
		let ovfl = byte_utils::bytes_add_bytes(
			self.as_bytes_mut(),
			&byte_utils::u32_to_bytes4(rhs)
		);
		assert!(!ovfl, "overflows should not occure for 256-bit keys");
	}
}

impl core::ops::Add<u64> for Key {
	type Output = Key;

	fn add(self, rhs: u64) -> Self::Output {
		let mut result = self;
		let ovfl = byte_utils::bytes_add_bytes(
			result.as_bytes_mut(),
			&byte_utils::u64_to_bytes8(rhs)
		);
		debug_assert!(!ovfl, "overflows should not occure for 256-bit keys");
		result
	}
}

impl core::ops::AddAssign<u64> for Key {
	fn add_assign(&mut self, rhs: u64) {
		let ovfl = byte_utils::bytes_add_bytes(
			self.as_bytes_mut(),
			&byte_utils::u64_to_bytes8(rhs)
		);
		debug_assert!(!ovfl, "overflows should not occure for 256-bit keys");
	}
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
	use super::*;

	use crate::env::{Env, ContractEnv};

	#[test]
	fn store_load_clear() {
		let key = Key([0x42; 32]);
		assert_eq!(unsafe { ContractEnv::load(key) }, None);
		unsafe { ContractEnv::store(key, &[0x5]); }
		assert_eq!(unsafe { ContractEnv::load(key) }, Some(vec![0x5]));
		unsafe { ContractEnv::clear(key); }
		assert_eq!(unsafe { ContractEnv::load(key) }, None);
	}

	#[test]
	fn key_with_offset() {
		let key00 = Key([0x0; 32]);
		let key05 = key00 + 5_u32;  // -> 5
		let key10 = key00 + 10_u32; // -> 10         | same as key55
		let key55 = key05 + 5_u32;  // -> 5 + 5 = 10 | same as key10
		unsafe { ContractEnv::store(key55, &[42]); }
		assert_eq!(unsafe { ContractEnv::load(key10) }, Some(vec![42]));
		unsafe { ContractEnv::store(key10, &[13, 37]); }
		assert_eq!(unsafe { ContractEnv::load(key55) }, Some(vec![13, 37]));
	}

	#[test]
	fn as_bytes() {
		let mut key = Key([0x42; 32]);
		assert_eq!(key.as_bytes(), &[0x42; 32]);
		assert_eq!(key.as_bytes_mut(), &mut [0x42; 32]);
	}

	#[test]
	fn key_diff() {
		let key1 = Key([0x0; 32]);
		let key2 = key1 + 0x42_u32;
		let key3 = key1 + u32::max_value() + 1_u32;
		let key4 = key1 + u64::max_value();
		assert_eq!(
			key2 - key1,
			KeyDiff([
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x42,
			])
		);
		assert_eq!((key2 - key1).try_to_u32(), Some(0x42));
		assert_eq!((key2 - key1).try_to_u64(), Some(0x42));
		assert_eq!(
			key3 - key1,
			KeyDiff([
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
			])
		);
		assert_eq!((key3 - key1).try_to_u32(), None);
		assert_eq!((key3 - key1).try_to_u64(), Some(u32::max_value() as u64 + 1));
		assert_eq!(
			key4 - key1,
			KeyDiff([
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
			])
		);
		assert_eq!((key4 - key1).try_to_u32(), None);
		assert_eq!(
			(key4 - key1).try_to_u64(),
			Some(u64::max_value())
		);
		assert_eq!(
			key4 - key3,
			KeyDiff([
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF,
			])
		);
		assert_eq!((key4 - key1).try_to_u32(), None);
		assert_eq!(
			(key4 - key3).try_to_u64(),
			Some(u64::max_value() - (u32::max_value() as u64 + 1))
		);
	}
}
