use super::*;
use crate::{
	storage::{
		self,
		Key,
	},
	byte_utils,
};

use parity_codec_derive::{Encode, Decode};

/// An allocator for the contract storage.
///
/// Specialized to efficiently allocate and deallocate cells and chunks.
///
/// # Note
///
/// This allocator allows for two types of allocations:
///
/// 1. Single cell allocation
/// 2. Cell chunk allocation (2^32 cells)
///
/// Allocating and deallocating are always O(1) operations.
#[derive(Debug, Encode, Decode)]
pub struct CellChunkAlloc {
	/// Allocator stash for single cells.
	cells: storage::Stash<()>,
	/// Allocator stash for cell chunks.
	chunks: storage::Stash<()>,
	/// Cells key offset.
	cells_off: storage::Key,
	/// Chunks key offset.
	chunks_off: storage::Key,
}

impl CellChunkAlloc {
	/// Creates a new cell & chunks allocator using the given allocator.
	///
	/// # Note
	///
	/// At first it might seem strange to initialize the one allocator
	/// with another. Normally a `CellChunkAllocator` should be allocated
	/// using a `ForwardAllocator`. The `ForwardAllocator` cannot be
	/// stored in the contract storage and is not useful for dynamic
	/// memory allocations but only for compile time allocations. The
	/// `CellChunkAllocator`, however, is made especially for the purpose
	/// of dynamic contract storage allocations and can and should be itself
	/// stored in the contract storage.
	pub unsafe fn new_using_alloc<A>(alloc: &mut A) -> Self
	where
		A: storage::Allocator
	{
		Self {
			cells: storage::Stash::new_using_alloc(alloc),
			chunks: storage::Stash::new_using_alloc(alloc),
			cells_off: alloc.alloc(u32::max_value()),
			chunks_off:
				// We need `u64::max_value()` here.
				// This depends on work on the Key API
				// to allow for `std::ops::Add<u64>`.
				//
				// As first iteration this should suffice our needs
				// as long as we allocate the `CellChunkAlloc` at last.
				alloc.alloc(u32::max_value()),
		}
	}

	/// Returns the key to the first cell allocation.
	///
	/// # Note
	///
	/// This key is then used to determine the key for every
	/// other cell allocation using its allocation index.
	pub(crate) fn cells_offset_key(&self) -> Key {
		self.cells_off
	}

	/// Returns the key to the first chunk allocation.
	///
	/// # Note
	///
	/// This key is then used to determine the key for every
	/// other chunk allocation using its allocation index.
	pub(crate) fn chunks_offset_key(&self) -> Key {
		self.chunks_off
	}

	/// Allocates a new storage region that fits for a single cell.
	fn alloc_cell(&mut self) -> Key {
		let index = self.cells.put(());
		self.cell_index_to_key(index)
	}

	/// Allocates a new storage region that fits for a whole chunk.
	fn alloc_chunk(&mut self) -> Key {
		let index = self.chunks.put(());
		self.chunk_index_to_key(index)
	}

	/// Deallocates a storage region fit for a single cell.
	fn dealloc_cell(&mut self, key: Key) {
		let index = self.key_to_cell_index(key);
		self.cells.take(index)
			.expect(
				"[pdsl_core::CellChunkAlloc::dealloc_cell] Error: \
				 key was not allocated by the allocator"
			)
	}

	/// Deallocates a storage region fit for a whole chunk.
	fn dealloc_chunk(&mut self, key: Key) {
		let index = self.key_to_chunk_index(key);
		self.chunks.take(index)
			.expect(
				"[pdsl_core::CellChunkAlloc::dealloc_chunk] Error: \
				 key was not allocated by the allocator"
			)
	}

	/// Converts cell indices to keys.
	///
	/// The reverse of `key_to_cell_index`.
	fn cell_index_to_key(&self, index: u32) -> Key {
		Key::with_offset(self.cells_offset_key(), index)
	}

	/// Converts keys to cell indices.
	///
	/// The reverse of `cell_index_to_key`.
	fn key_to_cell_index(&self, key: Key) -> u32 {
		let mut cell_offset = self.cells_offset_key();
		byte_utils::negate_bytes(cell_offset.as_bytes_mut());
		let mut res = key;
		byte_utils::bytes_add_bytes(res.as_bytes_mut(), cell_offset.as_bytes());
		debug_assert!(
			res.as_bytes()[0..28].into_iter().all(|&byte| byte == 0x0)
		);
		let index = byte_utils::bytes4_to_u32(
			byte_utils::slice4_as_array4(&res.as_bytes()[28..32])
				.unwrap()
		);
		index
	}

	/// Converts chunk indices to keys.
	///
	/// The reverse of `key_to_chunk_index`.
	fn chunk_index_to_key(&self, index: u32) -> Key {
		Key::with_chunk_offset(self.chunks_offset_key(), index)
	}

	/// Converts keys to chunk indices.
	///
	/// The reverse of `chunk_index_to_key`.
	fn key_to_chunk_index(&self, key: Key) -> u32 {
		let mut chunk_offset = self.chunks_offset_key();
		byte_utils::negate_bytes(chunk_offset.as_bytes_mut());
		let mut res = key;
		byte_utils::bytes_add_bytes(res.as_bytes_mut(), chunk_offset.as_bytes());
		debug_assert!(
			res.as_bytes()[0..24].into_iter().all(|&byte| byte == 0x0)
		);
		let index = byte_utils::bytes8_to_u64(
			byte_utils::slice8_as_array8(&res.as_bytes()[24..32])
				.unwrap()
		);
		(index >> 32) as u32
	}
}

impl Allocator for CellChunkAlloc {
	fn alloc(&mut self, size: u32) -> Key {
		debug_assert!(size != 0);
		if size <= 1 {
			self.alloc_cell()
		} else {
			self.alloc_chunk()
		}
	}

	fn dealloc(&mut self, key: Key) {
		// This assumes that the given key was previously
		// generated by the associated call to `Allocator::alloc`
		// of this same allocator implementor.
		assert!(key >= self.cells_offset_key());
		// This condition requires cells offset key
		// to be always smaller than chunks offset key.
		//
		// This must either be an invariant or we need
		// another more safe condition in the future.
		if key < self.chunks_offset_key() {
			// The key was allocated as a cell
			self.dealloc_cell(key)
		} else {
			// The key was allocated as a chunk
			self.dealloc_chunk(key)
		}
	}
}
