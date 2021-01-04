// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

//! Storage bit stash data structure and utilities.
//!
//! Allows to compactly and efficiently put and take bits in a compressed
//! and very efficient way.

mod counts;
mod storage;

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "ink-fuzz-tests"))]
mod fuzz_tests;

use self::counts::CountFree;
use crate::collections::{
    Bitvec as StorageBitvec,
    Vec as StorageVec,
};

/// The index type used in the storage bit stash.
type Index = u32;

/// A stash for bits operating on the contract storage.
///
/// Allows to efficiently put and take bits and
/// stores the underlying bits in an extremely compressed format.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BitStash {
    /// Counter for set bits in a 256-bit chunk of the `free` list.
    ///
    /// For every 256-bit chunk stored in `free` stores a `u8` that counts
    /// the number of set bits in the 256-bit chunk. This information is used
    /// to compact the information in `free` to make a `first fit` linear
    /// search for a new free storage slot more scalable. Since `u8` can only
    /// represent 256 different states but since we consider 0 we need an extra
    /// 9th bit. This 9th bit tells for every 256-bit chunk if it is full.
    ///
    /// In theory it is possible to search up to 8192 storage cells for free
    /// slots with a single contract storage look-up. By iterating over the 32
    /// `CountFree` instances of a single instance.
    counts: StorageVec<CountFree>,
    /// Stores the underlying bits of the storage bit stash.
    free: StorageBitvec,
}

impl BitStash {
    /// Creates a new storage bit stash.
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
        // Iterate over the `counts` list of the bit stash.
        // The counts list consists of packs of 32 counts per element.
        for (n, counts) in self.counts.iter_mut().enumerate() {
            if let Some(i) = counts.position_first_zero() {
                counts.inc(i as usize);
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

    /// Returns `true` if the bit at the indexed slot is set (`1`).
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn get(&self, index: Index) -> Option<bool> {
        self.free.get(index)
    }

    /// Puts another set bit into the storage bit stash.
    ///
    /// Returns the index to the slot where the set bit has been inserted.
    pub fn put(&mut self) -> Index {
        if let Some(index) = self.position_first_zero() {
            if index == self.free.len() as u64 {
                self.free.push(true);
                return self.free.len() - 1
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
                index as u32 + first_zero as u32
            } else {
                // We found a free storage slot but it isn't within the valid
                // bounds of the free list but points to its end. So we simply
                // append another 1 bit (`true`) to the free list and return
                // a new index pointing to it. No need to push to the counts
                // list in this case.
                self.free.push(true);
                self.free.len() - 1
            }
        } else {
            // We found no free 256-bit slot:
            //
            // - Check if we already have allocated too many (2^32) bits and
            // panic if that's the case. The check is done on the internal
            // storage bit vector.
            // - Otherwise allocate a new pack of 256-bits in the free list
            // and mirror it in the counts list.
            self.free.push(true);
            if self.counts.len() < self.required_counts() {
                // We need to push another counts element.
                let mut counter = CountFree::default();
                counter[0_u8] = 1;
                self.counts.push(counter);
            }
            // Return the new slot.
            self.free.len() - 1
        }
    }

    /// Takes the bit from the given index and returns it.
    ///
    /// Returns `true` if the indexed bit was set (`1`).
    /// Returns `None` if the index is out of bounds.
    ///
    /// # Note
    ///
    /// This frees up the indexed slot for putting in another set bit.
    pub fn take(&mut self, index: Index) -> Option<bool> {
        if index >= self.free.len() {
            // Bail out early if index is out of bounds.
            return None
        }
        let mut access = self.free.get_mut(index).expect("index is out of bounds");
        if !access.get() {
            return Some(false)
        }
        // At this point the bit was found to be set (`true`) and we have
        // update the underlying internals in order to reset it so the index
        // becomes free for another bit again.
        access.reset();
        // Update the counts list.
        let counts_id = index / (256 * 32);
        let byte_id = ((index / 256) % 32) as u8;
        self.counts
            .get_mut(counts_id)
            .expect("invalid counts ID")
            .dec(byte_id);
        Some(true)
    }
}
