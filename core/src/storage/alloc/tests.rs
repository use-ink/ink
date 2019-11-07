// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use super::*;

use crate::{
    storage::Key,
    test_utils::run_test,
};

#[test]
fn dyn_simple() {
    run_test(|| {
        use crate::storage;

        let mut alloc = unsafe {
            let mut fw_alloc = storage::alloc::BumpAlloc::from_raw_parts(Key([0x0; 32]));
            let mut dyn_alloc = storage::alloc::DynAlloc::allocate_using(&mut fw_alloc);
            dyn_alloc.initialize(());
            dyn_alloc
        };

        let cells_entries = dbg!(alloc.cells_origin());
        let chunks_entries = dbg!(alloc.chunks_origin());

        let mut cell_allocs = Vec::new();
        let mut chunk_allocs = Vec::new();

        // Cell allocations
        for i in 0..10 {
            let allocated_key = alloc.alloc(1);
            assert_eq!(allocated_key, cells_entries + (i as u32));
            cell_allocs.push(allocated_key);
        }

        // Chunk allocations
        let alloc_sizes = &[10, u32::max_value() as u64, 1337, 2, 9999_9999];
        for (i, &size) in alloc_sizes.into_iter().enumerate() {
            let allocated_key = alloc.alloc(size);
            assert_eq!(allocated_key, chunks_entries + ((1 << 32) * (i as u64)));
            chunk_allocs.push(allocated_key);
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
        assert_eq!(alloc.alloc(u32::max_value() as u64), chunk_allocs[0]);

        // Deallocate 2nd and 4th allocations in reverse order
        alloc.dealloc(chunk_allocs[3]);
        alloc.dealloc(chunk_allocs[1]);
        assert_eq!(alloc.alloc(u32::max_value() as u64), chunk_allocs[1]);
        assert_eq!(alloc.alloc(u32::max_value() as u64), chunk_allocs[3]);
    })
}
