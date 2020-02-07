// Copyright 2018-2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use ink_primitives::Key;

#[test]
fn dyn_simple() {
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
    for (i, &size) in alloc_sizes.iter().enumerate() {
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
}
