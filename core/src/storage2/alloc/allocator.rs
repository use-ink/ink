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
        let mut bits256_index: Option<u32> = None;
        let mut bits32_index: Option<u8> = None;
        // Iterate over the `counts` list of a dynamic allocator.
        // The counts list consists of packs of 32 counts per element.
        'outer: for (n, counts) in
            self.counts.iter_mut().map(Pack::as_inner_mut).enumerate()
        {
            // Iterate over the 32 `u8` within a single `CountFree` instance.
            for (i, count) in counts.iter_mut().enumerate() {
                if *count != 0xFF {
                    bits256_index = Some(n as u32);
                    bits32_index = Some(i as u8);
                    *count += 1;
                    break 'outer
                }
            }
        }
        if let (Some(bits256_index), Some(bits32_index)) = (bits256_index, bits32_index) {
            // At this point we know where we can find the next free slot in the
            // free list. We simply have to flag it there and return the value.
            // No need to update the set-bit counts list since that has already
            // happen at this point.
            let mut bits256 = self
                .free
                .get_mut(bits256_index)
                .expect("must exist if indices have been found");
            debug_assert!(!bits256.get());
            bits256.set();
            // TODO: We need to add an API to query 256-bit packages in order.
            todo!()
            // DynamicAllocation(bits256_index * 256 + bits32_index)
        } else {
            // We found no free dynamic storage slot:
            // Check if we already have allocated too many (2^32) dynamic
            // storage allocations and panic if that's the case.
            // Otherwise allocate a new pack of 256-bits in the free list
            // and mirror it in the counts list.
            let old_capacity = self.free.capacity();
            self.free.push(true);
            let new_capacity = self.free.capacity();
            if new_capacity > old_capacity {
                // A new 256-bit chunk has been allocated and there might be the
                // need to push another set-bit counts chunk as well:
                let q32x256 = 8192;
                if new_capacity / q32x256 > old_capacity / q32x256 {
                    let mut counter = CountFree::default();
                    counter[0_u8] = 1;
                    self.counts.push(Pack::from(counter));
                }
            }
            // Return the new slot.
            DynamicAllocation(self.free.len() - 1)
        }
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
        // TODO: Update the counts list.
        todo!()
    }
}

/// Stores the number of set bits for each 256-bits block in a compact `u8`.
#[derive(Debug, scale::Encode, scale::Decode)]
struct CountFree {
    counts: [u8; 32],
}

impl Default for CountFree {
    fn default() -> Self {
        Self::new()
    }
}

impl CountFree {
    /// Returns an iterator yielding shared references to the set-bit counts.
    pub fn iter(&self) -> core::slice::Iter<u8> {
        self.counts.iter()
    }

    /// Returns an iterator yielding exclusive references to the set-bit counts.
    pub fn iter_mut(&mut self) -> core::slice::IterMut<u8> {
        self.counts.iter_mut()
    }
}

impl<'a> IntoIterator for &'a CountFree {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut CountFree {
    type Item = &'a mut u8;
    type IntoIter = core::slice::IterMut<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
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

impl ::core::ops::Index<u8> for CountFree {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        &self.counts[index as usize]
    }
}

impl ::core::ops::IndexMut<u8> for CountFree {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.counts[index as usize]
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
