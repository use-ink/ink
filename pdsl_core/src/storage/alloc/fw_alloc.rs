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

use super::*;

use crate::{
	storage::Key,
};

/// An allocator that is meant to simply forward allocate contract
/// storage at compile-time.
///
/// # Note
///
/// It is not designed to be used during contract execution and it
/// also cannot deallocate key allocated by it.
///
/// Users are recommended to use the [`CellChunkAlloc`](struct.CellChunkAlloc.html)
/// for dynamic storage allocation purposes instead.
pub struct ForwardAlloc {
	/// The key offset used for all allocations.
	offset_key: Key,
}

impl ForwardAlloc {
	/// Creates a new forward allocator for the given raw parts.
	///
	/// # Note
	///
	/// Do not use this directly!
	/// This is meant to be used by pDSL internals only.
	pub unsafe fn from_raw_parts(offset_key: Key) -> Self {
		Self{
			offset_key,
		}
	}

	/// Increase the forward alloc offset key by the given amount.
	fn inc_offset_key(&mut self, by: u32) {
		self.offset_key += by;
	}
}

impl Allocator for ForwardAlloc {
	fn alloc(&mut self, size: u32) -> Key {
		if size == 0 {
			panic!(
				"[psdl_core::ForwardAlloc::alloc] Error: \
				 cannot allocate zero (0) bytes"
			)
		}
		let key = self.offset_key.clone();
		self.inc_offset_key(size);
		key
	}

	/// Not supported by this allocator!
	///
	/// Use [`CellChunkAlloc`](struct.CellChunkAlloc.html)
	/// for dynamic allocation purposes instead.
	fn dealloc(&mut self, _key: Key) {
		unreachable!(
			"The forward allocator is meant to be only used in compile-time
			 context for entities that shall not be deallocated during the
			 lifetime of a contract.\n\n Users are recommended to use the
			 `CellChunkAlloc` for dynamic.storage allocations instead."
		)
	}
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
	use super::*;

	#[test]
	fn allocate() {
		let offset_key = Key([0x00; 32]);
		let mut fw_alloc = unsafe {
			ForwardAlloc::from_raw_parts(offset_key)
		};
		assert_eq!(fw_alloc.alloc(1), offset_key + 0_u32);
		assert_eq!(fw_alloc.alloc(10), offset_key + 1_u32);
		assert_eq!(fw_alloc.alloc(u16::max_value() as u32), offset_key + 11_u32);
		assert_eq!(fw_alloc.alloc(2), offset_key + 0x1000A_u32);
		assert_eq!(fw_alloc.alloc(1), offset_key + 0x1000C_u32);
		assert_eq!(
			fw_alloc.alloc(u32::max_value()),
			offset_key + 0x1000D_u32,
		);
		assert_eq!(
			fw_alloc.alloc(1),
			Key([
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x0C,
			])
		)
	}

	#[test]
	#[should_panic]
	fn allocate_zero() {
		let offset_key = Key([0x00; 32]);
		let mut fw_alloc = unsafe {
			ForwardAlloc::from_raw_parts(offset_key)
		};
		fw_alloc.alloc(0);
	}

	#[test]
	#[should_panic]
	fn deallocate() {
		let offset_key = Key([0x00; 32]);
		let mut fw_alloc = unsafe {
			ForwardAlloc::from_raw_parts(offset_key)
		};
		let allocated_key = fw_alloc.alloc(1);
		assert_eq!(allocated_key, offset_key + 0_u32);
		fw_alloc.dealloc(allocated_key);
	}
}
