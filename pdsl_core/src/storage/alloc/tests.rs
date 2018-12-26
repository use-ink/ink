use super::*;

#[test]
fn simple() {
	use crate::storage;

	let cells_next_vacant = Key(
		[
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		]
	);
	let cells_len = Key::with_offset(cells_next_vacant, 1);
	let cells_entries = Key::with_offset(cells_len, u32::max_value());
	let chunks_next_vacant = Key(
		[
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		]
	);
	let chunks_len = Key::with_offset(chunks_next_vacant, 1);
	let chunks_entries = Key::with_offset(chunks_len, u32::max_value());
	let mut alloc = unsafe {
		CellChunkAlloc::from_raw_parts(
			storage::Stash::new_unchecked(
				cells_next_vacant,
				cells_len,
				cells_entries,
			),
			storage::Stash::new_unchecked(
				chunks_next_vacant,
				chunks_len,
				chunks_entries,
			),
		)
	};

	let mut cell_allocs = [Key([0; 32]); 5];
	let mut chunk_allocs = [Key([0; 32]); 5];

	// Cell allocations
	for i in 0..5 {
		cell_allocs[i] = alloc.alloc(1);
		assert_eq!(cell_allocs[i], Key::with_offset(cells_entries, i as u32));
	}

	// Chunk allocations
	let alloc_sizes = &[10, u32::max_value(), 1337, 2, 9999_9999];
	for (i, &size) in alloc_sizes.into_iter().enumerate() {
		chunk_allocs[i] = alloc.alloc(size);
		assert_eq!(chunk_allocs[i], Key::with_chunk_offset(chunks_entries, i as u32));
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
}