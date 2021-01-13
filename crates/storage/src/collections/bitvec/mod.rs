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

//! Storage bit vector data structure and utilities.
//!
//! Allows to compactly and efficiently store and manipulate on single bits.

mod bitref;
mod bits256;
mod bitsref;
mod impls;
mod iter;
mod storage;

#[cfg(test)]
mod tests;

pub use self::{
    bitref::BitRefMut,
    bitsref::ChunkRef,
    iter::{
        BitsIter,
        BitsIterMut,
    },
};
use self::{
    bits256::{
        Bits256,
        Iter as Bits256BitsIter,
        IterMut as Bits256BitsIterMut,
    },
    iter::{
        Bits256Iter,
        Bits256IterMut,
    },
};
use crate::{
    Lazy,
    Vec as StorageVec,
};

/// The index of a bit pack within the bit vector.
type Index = u32;

/// A bit position within a 256-bit package.
type Index256 = u8;

/// A bit position within a `u64`.
type Index64 = u8;

/// A pack of 64 bits.
type Bits64 = u64;

/// A storage bit vector.
///
/// # Note
///
/// Organizes its bits in chunks of 256 bits.
/// Allows to `push`, `pop`, inspect and manipulate the underlying bits.
#[derive(Debug)]
pub struct Bitvec {
    /// The length of the bit vector.
    len: Lazy<u32>,
    /// The bits of the bit vector.
    ///
    /// Organized in packs of 256 bits.
    bits: StorageVec<Bits256>,
}

impl Bitvec {
    /// Creates a new empty bit vector.
    pub fn new() -> Self {
        Self {
            len: Lazy::from(0),
            bits: StorageVec::new(),
        }
    }

    /// Returns the length of the bit vector in bits.
    pub fn len(&self) -> u32 {
        *self.len
    }

    /// Returns `true` if the bit vector is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the capacity of the bit vector in bits.
    ///
    /// # Note
    ///
    /// Returns a `u64` since it is always greater than or equal to `self.len()`
    /// which itself returns a `u32`.
    pub fn capacity(&self) -> u64 {
        (self.bits.len() * 256) as u64
    }

    /// Returns an iterator over the bits of the storage bit vector.
    pub fn bits(&self) -> BitsIter {
        BitsIter::new(self)
    }

    /// Returns an iterator over the mutable bits of the storage bit vector.
    pub fn bits_mut(&mut self) -> BitsIterMut {
        BitsIterMut::new(self)
    }

    /// Returns an iterator over the 256-bit chunks of the storage bit vector.
    pub(super) fn iter_chunks(&self) -> Bits256Iter {
        Bits256Iter::new(self)
    }

    /// Returns an iterator over the mutable 256-bit chunks of the storage bit vector.
    pub(super) fn iter_chunks_mut(&mut self) -> Bits256IterMut {
        Bits256IterMut::new(self)
    }

    /// Splits the given index into a 256-bit pack index and bit position index.
    fn split_index(&self, at: Index) -> Option<(Index, Index256)> {
        if at >= self.len() {
            return None
        }
        Some((at / 256, (at % 256) as u8))
    }

    /// Returns the immutable access pair to the underlying 256-bits pack and bit.
    ///
    /// Returns `None` if the given index is out of bounds.
    fn get_bits256(&self, at: Index) -> Option<(&Bits256, Index256)> {
        let (index, pos256) = self.split_index(at)?;
        let bits256 = self.bits.get(index).expect("index is out of bounds");
        Some((bits256, pos256))
    }

    /// Returns the mutable access pair to the underlying 256-bits pack and bit.
    ///
    /// Returns `None` if the given index is out of bounds.
    fn get_bits256_mut(&mut self, at: Index) -> Option<(&mut Bits256, Index256)> {
        let (index, pos256) = self.split_index(at)?;
        let bits256 = self.bits.get_mut(index).expect("index is out of bounds");
        Some((bits256, pos256))
    }

    /// Returns a mutable bit access to the bit at the given index if any.
    fn get_access_mut(&mut self, at: Index) -> Option<BitRefMut> {
        self.get_bits256_mut(at)
            .map(|(bits256, pos256)| BitRefMut::new(bits256, pos256))
    }

    /// Returns the value of the bit at the given index if any.
    pub fn get(&self, at: Index) -> Option<bool> {
        self.get_bits256(at)
            .map(|(bits256, pos256)| bits256.get(pos256))
    }

    /// Returns a mutable bit access to the bit at the given index if any.
    pub fn get_mut(&mut self, at: Index) -> Option<BitRefMut> {
        self.get_access_mut(at)
    }

    /// Returns a shared reference to the 256-bit chunk for the bit at the given index.
    pub fn get_chunk(&self, at: Index) -> Option<ChunkRef<&Bits256>> {
        if at >= self.len() {
            return None
        }
        use core::cmp::min;
        let chunk_id = at / 256;
        let chunk_len = min(256, self.len() - at);
        let bits256 = self.bits.get(chunk_id).expect("index is out of bounds");
        Some(ChunkRef::shared(bits256, chunk_len))
    }

    /// Returns an exclusive reference to the 256-bit chunk for the bit at the given index.
    pub fn get_chunk_mut(&mut self, at: Index) -> Option<ChunkRef<&mut Bits256>> {
        if at >= self.len() {
            return None
        }
        use core::cmp::min;
        let chunk_id = at / 256;
        let chunk_len = min(256, self.len() - at);
        let bits256 = self.bits.get_mut(chunk_id).expect("index is out of bounds");
        Some(ChunkRef::exclusive(bits256, chunk_len))
    }

    /// Returns the first bit of the bit vector.
    ///
    /// # Note
    ///
    /// Returns `None` if the bit vector is empty.
    pub fn first(&self) -> Option<bool> {
        if self.is_empty() {
            return None
        }
        self.get(0)
    }

    /// Returns a mutable bit access to the first bit of the bit vector.
    ///
    /// # Note
    ///
    /// Returns `None` if the bit vector is empty.
    pub fn first_mut(&mut self) -> Option<BitRefMut> {
        if self.is_empty() {
            return None
        }
        self.get_access_mut(0)
    }

    /// Returns the last bit of the bit vector.
    ///
    /// # Note
    ///
    /// Returns `None` if the bit vector is empty.
    pub fn last(&self) -> Option<bool> {
        if self.is_empty() {
            return None
        }
        self.get(self.len() - 1)
    }

    /// Returns a mutable bit access to the last bit of the bit vector.
    ///
    /// # Note
    ///
    /// Returns `None` if the bit vector is empty.
    pub fn last_mut(&mut self) -> Option<BitRefMut> {
        if self.is_empty() {
            return None
        }
        self.get_access_mut(self.len() - 1)
    }

    /// The maximum number of bits that can be pushed to a storage bit vector.
    fn maximum_capacity(&self) -> u32 {
        u32::MAX
    }

    /// Pushes the given value onto the bit vector.
    ///
    /// # Note
    ///
    /// This increases the length of the bit vector.
    ///
    /// # Panics
    ///
    /// If the storage bit vector reached its maximum capacity.
    pub fn push(&mut self, value: bool) {
        assert!(
            self.len() < self.maximum_capacity(),
            "reached maximum capacity for storage bit vector"
        );
        if self.len() as u64 == self.capacity() {
            // Case: All 256-bits packs are full or there are none:
            // Need to push another 256-bit pack to the storage vector.
            let mut bits256 = Bits256::default();
            if value {
                // If `value` is `true` set its first bit to `1`.
                bits256.set(0);
                debug_assert_eq!(bits256.get(0), true);
            };
            self.bits.push(bits256);
            *self.len += 1;
        } else {
            // Case: The last 256-bit pack has unused bits:
            // - Set last bit of last 256-bit pack to the given value.
            // - Opt.: Since bits are initialized as 0 we only need
            //         to mutate this value if `value` is `true`.
            *self.len += 1;
            if value {
                self.last_mut()
                    .expect("must have at least a valid bit in this case")
                    .set()
            }
        }
    }

    /// Pops the last bit from the bit vector.
    ///
    /// Returns the popped bit as `bool`.
    ///
    /// # Note
    ///
    /// This reduces the length of the bit vector by one.
    pub fn pop(&mut self) -> Option<bool> {
        if self.is_empty() {
            // Bail out early if the bit vector is emtpy.
            return None
        }
        let mut access = self.last_mut().expect("must be some if non-empty");
        let popped = access.get();
        access.reset();
        *self.len -= 1;
        Some(popped)
    }
}
