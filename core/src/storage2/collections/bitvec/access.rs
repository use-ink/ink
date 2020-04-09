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

use super::{
    Bits256,
    Index256,
    Bits256BitsIter,
    Bits256BitsIterMut,
};

/// A mutable bit access for operating on a single bit within a 256-bit pack.
#[derive(Debug)]
pub struct BitRefMut<'a> {
    /// The queried pack of 256 bits.
    bits: &'a mut Bits256,
    /// The bit position witihn the queried bit pack.
    at: u8,
}

impl<'a> BitRefMut<'a> {
    /// Creates a new bit access for the indexed bit within the 256-bit pack.
    pub(super) fn new(bits: &'a mut Bits256, at: Index256) -> Self {
        Self { bits, at }
    }

    /// Returns the value of the indexed bit.
    ///
    /// # Note
    ///
    /// - If 0: returns `false`
    /// - If 1: returns `true`
    pub fn get(&self) -> bool {
        self.bits.get(self.at)
    }

    /// Sets the value of the indexed bit to the given new value.
    pub fn set_to(&mut self, new_value: bool) {
        self.bits.set_to(self.at, new_value)
    }

    /// Sets the indexed bit to `1` (true).
    pub fn set(&mut self) {
        self.bits.set(self.at)
    }

    /// Resets the indexed bit to `0` (false).
    pub fn reset(&mut self) {
        self.bits.set(self.at)
    }

    /// Flips the indexed bit.
    pub fn flip(&mut self) {
        self.bits.flip(self.at)
    }

    /// Computes bitwise XOR for the indexed bit and `rhs`.
    pub fn xor(&mut self, rhs: bool) {
        self.bits.xor(self.at, rhs)
    }

    /// Computes bitwise AND for the indexed bit and `rhs`.
    pub fn and(&mut self, rhs: bool) {
        self.bits.and(self.at, rhs)
    }

    /// Computes bitwise OR for the indexed bit and `rhs`.
    pub fn or(&mut self, rhs: bool) {
        self.bits.or(self.at, rhs)
    }
}

/// A mutable chunk of up to 256 bits.
///
/// # Note
///
/// - Allows to inspect and manipulate bits in the 256-bit chunk on a bitwise and
///   on chunk level.
/// - Assures that accesses to the underlying bits are valid for the storage
///   bit vector in which they are stored.
#[derive(Debug)]
pub struct Bits256RefMut<'a> {
    /// The queried pack of 256 bits.
    chunk: &'a mut Bits256,
    /// The length of the chunk.
    ///
    /// This is the number of valid bits in the chunk of 256 bits.
    /// The valid bits are consecutive and always start from index 0.
    len: u32,
}

impl<'a> Bits256RefMut<'a> {
    /// Creates a new 256-bit chunk access with the given length.
    pub(super) fn new(chunk: &'a mut Bits256, len: u32) -> Self {
        Self { chunk, len }
    }

    /// Returns the length of the 256-bit chunk.
    ///
    /// # Note
    ///
    /// This is the number of valid bits in the chunk of 256 bits.
    /// The valid bits are consecutive and always start from index 0.
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Returns an iterator over the valid bits of `self`.
    pub fn iter(&self) -> Bits256BitsIter {
        self.chunk.iter(self.len as u16)
    }

    /// Returns an iterator over mutable accessors to the valid bits of `self`.
    pub fn iter_mut(&mut self) -> Bits256BitsIterMut {
        self.chunk.iter_mut(self.len as u16)
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
        self.chunk.get(index).into()
    }

    /// Returns mutable access to a single bit if the index is within bounds.
    pub fn get_mut(&mut self, index: u8) -> Option<BitRefMut> {
        if index as u32 >= self.len {
            return None
        }
        BitRefMut::new(self.chunk, index).into()
    }
}
    }

    /// Returns the position of the first valid zero bit if any.
    pub fn position_first_zero(&self) -> Option<u8> {
        let position = self.chunk.position_first_zero()?;
        if position as u32 >= self.len() {
            return None
        }
        Some(position)
    }
}
