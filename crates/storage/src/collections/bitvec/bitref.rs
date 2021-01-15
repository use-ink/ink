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

#![allow(clippy::len_without_is_empty)]

use super::{
    Bits256,
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

impl<'a> PartialEq for BitRefMut<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<'a> Eq for BitRefMut<'a> {}

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
mod tests {
    use super::BitRefMut;
    use crate::collections::bitvec::Bits256;

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
