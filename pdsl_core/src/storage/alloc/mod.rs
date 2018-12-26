//! Facilities to allocate and deallocate contract storage dynamically.

mod cc_alloc;
mod fw_alloc;

#[cfg(all(test, feature = "test-env"))]
mod tests;

use crate::storage::Key;

pub use self::{
	cc_alloc::{
		CellChunkAlloc,
	},
	fw_alloc::{
		ForwardAlloc,
	},
};

/// Types implementing this trait are storage allocators.
pub trait Allocator {
	/// Allocates a storage area.
	///
	/// The returned key denotes a storage region that fits for at
	/// least the given number of cells.
	fn alloc(&mut self, size: u32) -> Key;

	/// Deallocates a storage area.
	///
	/// The given storage region must have been allocated by this
	/// allocator before.
	fn dealloc(&mut self, key: Key);
}
