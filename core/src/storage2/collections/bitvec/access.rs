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
    Bits256,
    Bits256BitsIter,
    Bits256BitsIterMut,
    Index256,
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
        self.bits.reset(self.at)
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

#[cfg(test)]
mod bit_ref_mut_tests {
    use super::BitRefMut;
    use crate::storage2::collections::bitvec::Bits256;

    fn is_populated_bit_set(index: u8) -> bool {
        (index % 5) == 0 || (index % 13) == 0
    }

    fn populated_bits256() -> Bits256 {
        let mut bits256 = Bits256::default();
        for i in 0..256 {
            let i = i as u8;
            bits256.set_to(i, is_populated_bit_set(i));
        }
        bits256
    }

    #[test]
    fn get_set_works() {
        let mut bits256 = populated_bits256();
        for i in 0..=255 {
            let mut bitref = BitRefMut::new(&mut bits256, i);
            let expected = is_populated_bit_set(i);
            assert_eq!(bitref.get(), expected);
            // Set only every second bit to true and check this later:
            bitref.set_to(i % 2 == 0);
        }
        // Check if `set_to` was successful:
        for i in 0..=255 {
            assert_eq!(bits256.get(i), i % 2 == 0);
        }
    }

    #[test]
    fn flip_works() {
        let mut bits256 = populated_bits256();
        for i in 0..=255 {
            let mut bitref = BitRefMut::new(&mut bits256, i);
            bitref.flip();
        }
        // Check if `flip` was successful:
        for i in 0..=255 {
            assert_eq!(bits256.get(i), !is_populated_bit_set(i));
        }
    }

    #[test]
    fn set_and_reset_works() {
        let mut bits256 = populated_bits256();
        for i in 0..=255 {
            let mut bitref = BitRefMut::new(&mut bits256, i);
            if i % 2 == 0 {
                bitref.set();
            } else {
                bitref.reset();
            }
        }
        // Check if `set` and `reset` was successful:
        for i in 0..=255 {
            assert_eq!(bits256.get(i), i % 2 == 0);
        }
    }

    #[test]
    fn bitops_works() {
        let mut bits256 = populated_bits256();
        for i in 0..=255 {
            let mut bitref = BitRefMut::new(&mut bits256, i);
            let expected = is_populated_bit_set(i);
            fn test_xor(bitref: &mut BitRefMut, expected: bool) {
                fn test_xor_for(bitref: &mut BitRefMut, expected: bool, input: bool) {
                    assert_eq!(bitref.get(), expected);
                    bitref.xor(input);
                    assert_eq!(bitref.get(), expected ^ input);
                    bitref.set_to(expected);
                }
                test_xor_for(bitref, expected, false);
                test_xor_for(bitref, expected, true);
            }
            test_xor(&mut bitref, expected);
            fn test_and(bitref: &mut BitRefMut, expected: bool) {
                fn test_and_for(bitref: &mut BitRefMut, expected: bool, input: bool) {
                    assert_eq!(bitref.get(), expected);
                    bitref.and(input);
                    assert_eq!(bitref.get(), expected & input);
                    bitref.set_to(expected);
                }
                test_and_for(bitref, expected, false);
                test_and_for(bitref, expected, true);
            }
            test_and(&mut bitref, expected);
            fn test_or(bitref: &mut BitRefMut, expected: bool) {
                fn test_or_for(bitref: &mut BitRefMut, expected: bool, input: bool) {
                    assert_eq!(bitref.get(), expected);
                    bitref.or(input);
                    assert_eq!(bitref.get(), expected | input);
                    bitref.set_to(expected);
                }
                test_or_for(bitref, expected, false);
                test_or_for(bitref, expected, true);
            }
            test_or(&mut bitref, expected);
        }
    }
}

/// A reference to a subslice within a 256-bit chunk.
///
/// This is a reference wrapper around either a shared 256-bit chunk
/// or an exclusive 256-bit chunk. Also it prevents accesses to out of bounds
/// bits.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
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
        unsafe { core::mem::transmute::<&Self, &Self::Target>(self) }
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
