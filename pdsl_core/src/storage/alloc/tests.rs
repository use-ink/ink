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

use crate::test_utils::run_test;

#[test]
fn simple() {
	run_test(|| {
		use crate::storage;

		let mut alloc = unsafe {
			let mut fw_alloc = storage::alloc::ForwardAlloc::from_raw_parts(
				Key([0x0; 32])
			);
			storage::alloc::CellChunkAlloc::new_using_alloc(&mut fw_alloc)
		};

		let cells_entries = alloc.cells_offset_key();
		let chunks_entries = alloc.chunks_offset_key();

		let mut cell_allocs = [Key([0; 32]); 5];
		let mut chunk_allocs = [Key([0; 32]); 5];

		// Cell allocations
		for i in 0..5 {
			cell_allocs[i] = alloc.alloc(1);
			assert_eq!(cell_allocs[i], cells_entries + (i as u32));
		}

		// Chunk allocations
		let alloc_sizes = &[10, u32::max_value(), 1337, 2, 9999_9999];
		for (i, &size) in alloc_sizes.into_iter().enumerate() {
			chunk_allocs[i] = alloc.alloc(size);
			assert_eq!(chunk_allocs[i], chunks_entries + ((1 << 32) * (i as u64)));
		}

		// Deallocate first cell again
		alloc.dealloc(cell_allocs[0]);
		// Now the next cell allocation will take the first allocation cell again
		assert_eq!(alloc.alloc(1), cell_allocs[0]);

		// Deallocate 2nd and 4th allocations in reverse order
		alloc.dealloc(cell_allocs[3]);
		alloc.dealloc(cell_allocs[1]);
		assert_eq!(alloc.alloc(1), cell_allocs[1]);
		assert_eq!(alloc.alloc(1), cell_allocs[3]);

		// Deallocate first chunk again
		alloc.dealloc(chunk_allocs[0]);
		// Now the next chunk allocation will take the first allocation cell again
		assert_eq!(alloc.alloc(u32::max_value()), chunk_allocs[0]);

		// Deallocate 2nd and 4th allocations in reverse order
		alloc.dealloc(chunk_allocs[3]);
		alloc.dealloc(chunk_allocs[1]);
		assert_eq!(alloc.alloc(u32::max_value()), chunk_allocs[1]);
		assert_eq!(alloc.alloc(u32::max_value()), chunk_allocs[3]);
	})
}
