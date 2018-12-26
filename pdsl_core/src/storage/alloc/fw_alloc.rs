use super::*;

use crate::{
	storage::Key,
};

use std::sync::atomic::{
	AtomicUsize,
	Ordering,
};

/// An allocator that is meant to simply forward allocate contract
/// storage at compile-time.
///
/// # Note
///
/// It is not designed to be used during contract execution and it
/// also cannot deallocate key allocated by it.
///
/// Users are recommended to use the `CellChunkAlloc` for dynamic
/// storage allocation purposes instead.
pub struct ForwardAlloc {
	/// The key offset used for all allocations.
	offset_key: Key,
	/// The offset added to the key offset for the next allocation.
	offset_idx: AtomicUsize,
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
			offset_idx: AtomicUsize::new(0)
		}
	}
}

impl Allocator for ForwardAlloc {
	fn alloc(&mut self, size: u32) -> Key {
		let next_idx = self.offset_idx.fetch_add(size as usize, Ordering::SeqCst);
		if next_idx >= u32::max_value() as usize {
			panic!(
				"[pdsl_core::ForwardAlloc::alloc] Error: \
				 cannot allocate more than u32::MAX entities"
			)
		}
		Key::with_offset(self.offset_key, next_idx as u32)
	}

	/// Not supported by this allocator!
	///
	/// Use `CellChunkAlloc` for dynamic allocation purposes instead.
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
		assert_eq!(fw_alloc.alloc(1), Key::with_offset(offset_key, 0));
		assert_eq!(fw_alloc.alloc(10), Key::with_offset(offset_key, 1));
		assert_eq!(fw_alloc.alloc(u16::max_value() as u32), Key::with_offset(offset_key, 11));
		assert_eq!(fw_alloc.alloc(2), Key::with_offset(offset_key, 0x1000A));
		assert_eq!(fw_alloc.alloc(1), Key::with_offset(offset_key, 0x1000C));
	}

	#[test]
	#[should_panic]
	fn deallocate() {
		let offset_key = Key([0x00; 32]);
		let mut fw_alloc = unsafe {
			ForwardAlloc::from_raw_parts(offset_key)
		};
		let allocated_key = fw_alloc.alloc(1);
		assert_eq!(allocated_key, Key::with_offset(offset_key, 0));
		fw_alloc.dealloc(allocated_key);
	}
}
