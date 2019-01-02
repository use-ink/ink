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
