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
    /// Creates a new dynamic storage allocator.
    pub fn new() -> Self {
        Self {
            counts: StorageVec::new(),
            free: StorageBitvec::new(),
        }
    }

    /// Returns the bit position of the first 256-bit chunk with zero bits
    /// in the `free` list.
    ///
    /// Returns the bit position of the first bit in the 256-bit chunk and not
    /// the chunk position since that's what [`Bitvec::get_chunk`] expects.
    ///
    /// Also directly increases the count of the first found free bit chunk.
    fn position_first_zero(&mut self) -> Option<u64> {
        // Iterate over the `counts` list of a dynamic allocator.
        // The counts list consists of packs of 32 counts per element.
        for (n, counts) in self.counts.iter_mut().map(Pack::as_inner_mut).enumerate() {
            if let Some(i) = counts.position_first_zero() {
                let n = n as u64;
                let i = i as u64;
                return Some(n * (32 * 256) + i * 256)
            }
        }
        None
    }

    /// Returns the number of required counts elements.
    fn required_counts(&self) -> u32 {
        let capacity = self.free.capacity();
        if capacity == 0 {
            return 0
        }
        1 + ((capacity - 1) / (32 * 256)) as u32
    }

    /// Returns a new dynamic storage allocation.
    ///
    /// # Panics
    ///
    /// If the dynamic allocator ran out of free dynamic allocations.
    pub fn alloc(&mut self) -> DynamicAllocation {
        if let Some(index) = self.position_first_zero() {
            // At this point we know where we can find the next free slot in the
            // free list. We simply have to flag it there and return the value.
            // No need to update the set-bit counts list since that has already
            // happen at this point.
            let mut bits256 = self
                .free
                .get_chunk_mut(index as u32)
                .expect("must exist if indices have been found");
            let first_zero = bits256
                .position_first_zero()
                .expect("must exist if counts told so");
            bits256
                .get_mut(first_zero)
                .expect("first zero is invalid")
                .set();
            DynamicAllocation(index as u32 + first_zero as u32)
        } else {
            // We found no free dynamic storage slot:
            // Check if we already have allocated too many (2^32) dynamic
            // storage allocations and panic if that's the case.
            // Otherwise allocate a new pack of 256-bits in the free list
            // and mirror it in the counts list.
            self.free.push(true);
            if self.counts.len() < self.required_counts() {
                // We need to push another counts element.
                let mut counter = CountFree::default();
                counter[0_u8] = 1;
                self.counts.push(Pack::from(counter));
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
        let index = allocation.get();
        let mut access = self
            .free
            .get_mut(index)
            .expect("index is out of bounds");
        // Panic if the given dynamic allocation is not represented as
        // occupied in the `free` list.
        assert!(access.get());
        // Set to `0` (false) which means that this slot is available again.
        access.reset();
        // Update the counts list.
        let counts_id = index / (256 * 32);
        let byte_id = (index % 32) as u8;
        self.counts.get_mut(counts_id).expect("invalid counts ID").dec(byte_id);
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

    /// Returns the position of the first free `u8` in the free counts.
    ///
    /// Returns `None` if all counts are `0xFF`.
    pub fn position_first_zero(&mut self) -> Option<u8> {
        for (i, count) in self.counts.iter_mut().enumerate() {
            if *count != 0xFF {
                *count += 1;
                return Some(i as u8)
            }
        }
        None
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
            .checked_sub(1)
            .expect("set bits decrement overflowed");
        self.counts[index as usize] = new_value;
        new_value
    }
}
