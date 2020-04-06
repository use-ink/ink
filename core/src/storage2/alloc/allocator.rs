// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

use super::DynamicAllocation;
use crate::storage2::{
    collections::Bitvec as StorageBitvec,
    KeyPtr,
    Pack,
    PullAt,
    PullForward,
    PushAt,
    PushForward,
    StorageFootprint,
    Vec as StorageVec,
};
use ink_primitives::Key;

/// An index into the dynamic allocator's free list.
type Index = u32;

/// An index into one of the 32 `SetBit32` entries.
type Index32 = u8;

/// The dynamic allocator.
///
/// Manages dynamic storage allocations in a very efficient and economic way.
pub struct DynamicAllocator {
    /// Counter for set bits in a 256-bit chunk of the `free` list.
    ///
    /// For every 256-bit chunk stored in `free` stores a `u8` that counts
    /// the number of set bits in the 256-bit chunk. This information is used
    /// to compact the information in `free` to make a `first fit` linear
    /// search for a new free storage slot more scalable.
    ///
    /// In theory it is possible to search up to 8192 storage cells for free
    /// slots with a single contract storage look-up. By iterating over the 32
    /// `SetBits32` instances of a single instance.
    counts: StorageVec<Pack<CountFree>>,
    /// Stores a bit for every allocated or free storage cell.
    free: StorageBitvec,
}

impl StorageFootprint for DynamicAllocator {
    const VALUE: u64 = <StorageVec<Pack<CountFree>> as StorageFootprint>::VALUE
        + <StorageBitvec as StorageFootprint>::VALUE;
}

impl PullForward for DynamicAllocator {
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        Self {
            counts: PullForward::pull_forward(ptr),
            free: PullForward::pull_forward(ptr),
        }
    }
}

impl PushForward for DynamicAllocator {
    fn push_forward(&self, ptr: &mut KeyPtr) {
        PushForward::push_forward(&self.counts, ptr);
        PushForward::push_forward(&self.free, ptr);
    }
}

impl DynamicAllocator {
    /// Returns a new dynamic storage allocation.
    ///
    /// # Panics
    ///
    /// If the dynamic allocator ran out of free dynamic allocations.
    pub fn alloc(&mut self) -> DynamicAllocation {
        todo!()
    }

    /// Frees the given dynamic storage allocation.
    ///
    /// This makes the given dynamic storage allocation available again
    /// for new dynamic storage allocations.
    ///
    /// # Panics
    ///
    /// Panics if the given dynamic allocation is invalid.
    /// A dynamic allocation is invalid if it is not represented as occupied
    /// in the `free` list.
    pub fn free(&mut self, allocation: DynamicAllocation) {
        let mut access = self
            .free
            .get_mut(allocation.get())
            .expect("index is out of bounds");
        // Panic if the given dynamic allocation is not represented as
        // occupied in the `free` list.
        assert!(access.get());
        // Set to `0` (false) which means that this slot is available again.
        access.reset();
    }
}

/// Stores the number of set bits for each 256-bits block in a compact `u8`.
#[derive(Debug, scale::Encode, scale::Decode)]
struct CountFree {
    counts: [u8; 32],
}

impl PullAt for CountFree {
    fn pull_at(at: Key) -> Self {
        Self {
            counts: <[u8; 32] as PullAt>::pull_at(at),
        }
    }
}

impl PushAt for CountFree {
    fn push_at(&self, at: Key) {
        PushAt::push_at(&self.counts, at);
    }
}

impl CountFree {
    /// Creates a new 32-entity set bit counter initialized with zeros.
    pub fn new() -> Self {
        Self { counts: [0x00; 32] }
    }

    /// Returns the number of set bits for the given index.
    ///
    /// # Panics
    ///
    /// - If the given index is out of bounds.
    pub fn get(&self, index: Index32) -> u8 {
        assert!(index < 32, "index is out of bounds");
        self.counts[index as usize]
    }

    /// Increases the number of set bits for the given index.
    ///
    /// Returns the new number of set bits.
    ///
    /// # Panics
    ///
    /// - If the given index is out of bounds.
    /// - If the increment would cause an overflow.
    pub fn inc(&mut self, index: Index32) -> u8 {
        assert!(index < 32, "index is out of bounds");
        let new_value = self.counts[index as usize]
            .checked_add(1)
            .expect("set bits increment overflowed");
        self.counts[index as usize] = new_value;
        new_value
    }

    /// Increases the number of set bits for the given index.
    ///
    /// Returns the new number of set bits.
    ///
    /// # Panics
    ///
    /// - If the given index is out of bounds.
    /// - If the increment would cause an overflow.
    pub fn dec(&mut self, index: Index32) -> u8 {
        assert!(index < 32, "index is out of bounds");
        let new_value = self.counts[index as usize]
            .checked_add(1)
            .expect("set bits decrement overflowed");
        self.counts[index as usize] = new_value;
        new_value
    }
}
