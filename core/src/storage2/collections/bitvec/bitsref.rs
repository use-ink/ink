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

#![allow(clippy::len_without_is_empty)]

use super::{
    BitRefMut,
    Bits256,
    Bits256BitsIter,
    Bits256BitsIterMut,
};

/// A reference to a subslice within a 256-bit chunk.
///
/// This is a reference wrapper around either a shared 256-bit chunk
/// or an exclusive 256-bit chunk. Also it prevents accesses to out of bounds
/// bits.
#[derive(Debug, Copy, Clone)]
#[repr(C)] // This is repr(C) to be on the safe side for the Deref impl.
pub struct ChunkRef<T> {
    /// The reference to the 256-bits chunk.
    bits: T,
    /// The length of the accessible chunk area.
    len: u32,
}

impl<'a> ChunkRef<&'a Bits256> {
    /// Creates a new shared 256-bit chunk access with the given length.
    pub(super) fn shared(bits: &'a Bits256, len: u32) -> Self {
        Self { bits, len }
    }

    /// Returns the position of the first valid zero bit if any.
    pub fn position_first_zero(&self) -> Option<u8> {
        let position = self.bits.position_first_zero()?;
        if position as u32 >= self.len() {
            return None
        }
        Some(position)
    }

    /// Returns the value of the indexed bit.
    ///
    /// # Note
    ///
    /// - If 0: returns `false`
    /// - If 1: returns `true`
    pub fn get(&self, index: u8) -> Option<bool> {
        if index as u32 >= self.len {
            return None
        }
        self.bits.get(index).into()
    }

    /// Returns an iterator over the valid bits of `self`.
    pub(super) fn iter(&self) -> Bits256BitsIter {
        self.bits.iter(self.len as u16)
    }
}

impl<'a> core::ops::Deref for ChunkRef<&'a mut Bits256> {
    type Target = ChunkRef<&'a Bits256>;

    fn deref(&self) -> &Self::Target {
        // This implementation allows to mirror the interface on
        // `ChunkRef<&'a Bits256>` onto `ChunkRef<&'a mut Bits256>`
        // without the need of separate implementations.
        //
        // SAFETY: The `ChunkRef` struct is `repr(C)` which should guarantee
        //         that both `ChunkRef<&'a mut Bits256>` as well as
        //         `ChunkRef<&'a Bits256>` have the same internal layout
        //         and thus can be transmuted safely.
        let ptr: *const Self = self;
        unsafe { &*(ptr as *const Self::Target) }
    }
}

impl<'a> ChunkRef<&'a mut Bits256> {
    /// Creates a new exclusive 256-bit chunk access with the given length.
    pub(super) fn exclusive(bits: &'a mut Bits256, len: u32) -> Self {
        Self { bits, len }
    }

    /// Returns mutable access to a single bit if the index is within bounds.
    pub fn get_mut(&mut self, index: u8) -> Option<BitRefMut> {
        if index as u32 >= self.len {
            return None
        }
        BitRefMut::new(self.bits, index).into()
    }

    /// Returns an iterator over mutable accessors to the valid bits of `self`.
    pub(super) fn iter_mut(&mut self) -> Bits256BitsIterMut {
        self.bits.iter_mut(self.len as u16)
    }
}

impl<T> ChunkRef<T> {
    /// Returns the length of the 256-bit chunk.
    ///
    /// # Note
    ///
    /// This is the number of valid bits in the chunk of 256 bits.
    /// The valid bits are consecutive and always start from index 0.
    pub fn len(&self) -> u32 {
        self.len
    }
}
