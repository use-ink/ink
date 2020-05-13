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
    traits::{
        forward_clear_packed,
        forward_pull_packed,
        forward_push_packed,
        KeyPtr,
        PackedLayout,
        SpreadLayout,
    },
    Pack,
    Vec as StorageVec,
};
use ink_primitives::Key;

/// An index into one of the 32 `SetBit32` entries.
type Index32 = u8;

/// The dynamic allocator.
///
/// Manages dynamic storage allocations in a very efficient and economic way.
#[derive(Debug)]
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

impl SpreadLayout for CountFree {
    const FOOTPRINT: u64 = 1;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        forward_pull_packed::<Self>(ptr)
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        forward_push_packed::<Self>(self, ptr)
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        forward_clear_packed::<Self>(self, ptr)
    }
}

impl PackedLayout for CountFree {
    fn pull_packed(&mut self, _at: &Key) {}
    fn push_packed(&self, _at: &Key) {}
    fn clear_packed(&self, _at: &Key) {}
}

impl SpreadLayout for DynamicAllocator {
    const FOOTPRINT: u64 = <StorageVec<Pack<CountFree>> as SpreadLayout>::FOOTPRINT
        + <StorageBitvec as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Self {
            counts: SpreadLayout::pull_spread(ptr),
            free: SpreadLayout::pull_spread(ptr),
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::push_spread(&self.counts, ptr);
        SpreadLayout::push_spread(&self.free, ptr);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::clear_spread(&self.counts, ptr);
        SpreadLayout::clear_spread(&self.free, ptr);
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

    /// Resets the dynamic allocator.
    ///
    /// # Note
    ///
    /// This method is only needed for testing when tools such as `miri` run all
    /// tests on the same thread in sequence so every test has to reset the
    /// statically allocated instances like the storage allocator before running.
    #[cfg(test)]
    pub fn reset(&mut self) {
        self.counts = StorageVec::new();
        self.free = StorageBitvec::new();
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
            if index == self.free.len() as u64 {
                self.free.push(true);
                return DynamicAllocation(self.free.len() - 1)
            }
            let mut bits256 = self
                .free
                .get_chunk_mut(index as u32)
                .expect("must exist if indices have been found");
            if let Some(first_zero) = bits256.position_first_zero() {
                bits256
                    .get_mut(first_zero)
                    .expect("first zero is invalid")
                    .set();
                DynamicAllocation(index as u32 + first_zero as u32)
            } else {
                // We found a free storage slot but it isn't within the valid
                // bounds of the free list but points to its end. So we simply
                // append another 1 bit (`true`) to the free list and return
                // a new dynamic allocation pointing to it. No need to push to
                // the counts list in this case.
                self.free.push(true);
                DynamicAllocation(self.free.len() - 1)
            }
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
        let mut access = self.free.get_mut(index).expect("index is out of bounds");
        // Panic if the given dynamic allocation is not represented as
        // occupied in the `free` list.
        assert!(access.get(), "encountered double free of dynamic storage");
        // Set to `0` (false) which means that this slot is available again.
        access.reset();
        // Update the counts list.
        let counts_id = index / (256 * 32);
        let byte_id = ((index / 256) % 32) as u8;
        self.counts
            .get_mut(counts_id)
            .expect("invalid counts ID")
            .dec(byte_id);
    }
}

/// Stores the number of set bits for each 256-bits block in a compact `u8`.
#[derive(Debug, scale::Encode, scale::Decode)]
struct CountFree {
    /// Set bits per 256-bit chunk.
    counts: [u8; 32],
    /// Since with `u8` can only count up to 255 but there might be the need
    /// to count up to 256 bits for 256-bit chunks we need to store one extra
    /// bit per counter to determine filled chunks.
    full: FullMask,
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

#[derive(Debug, Copy, Clone, scale::Encode, scale::Decode)]
pub struct FullMask(u32);

impl Default for FullMask {
    fn default() -> Self {
        Self::new()
    }
}

impl FullMask {
    /// Creates a new full mask with every flag set to `false`.
    pub fn new() -> Self {
        Self(0)
    }

    /// Returns `true` if the 256-bit chunk at the given index is full.
    pub fn is_full(self, index: u8) -> bool {
        assert!(index < 32);
        (self.0 >> (31 - index as u32)) & 0x01 == 1
    }

    /// Sets the flag for the 256-bit chunk at the given index to `full`.
    pub fn set_full(&mut self, index: u8) {
        self.0 |= 1_u32 << (31 - index as u32);
    }

    /// Resets the flag for the 256-bit chunk at the given index to not `full`.
    pub fn reset_full(&mut self, index: u8) {
        self.0 &= !(1_u32 << (31 - index as u32));
    }
}

impl CountFree {
    /// Creates a new 32-entity set bit counter initialized with zeros.
    pub fn new() -> Self {
        Self {
            counts: Default::default(),
            full: Default::default(),
        }
    }

    /// Returns the position of the first free `u8` in the free counts.
    ///
    /// Returns `None` if all counts are `0xFF`.
    pub fn position_first_zero(&mut self) -> Option<u8> {
        for (i, count) in self.counts.iter_mut().enumerate() {
            if !self.full.is_full(i as u8) {
                if *count == !0 {
                    self.full.set_full(i as u8);
                } else {
                    *count += 1;
                }
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
    pub fn dec(&mut self, index: Index32) -> u8 {
        assert!(index < 32, "index is out of bounds");
        if self.full.is_full(index) {
            self.full.reset_full(index);
        } else {
            let new_value = self.counts[index as usize]
                .checked_sub(1)
                .expect("set bits decrement overflowed");
            self.counts[index as usize] = new_value;
        }
        self.counts[index as usize]
    }
}
