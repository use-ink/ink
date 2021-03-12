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

use super::{
    super::extend_lifetime,
    BitRefMut,
    Bits64,
    Index256,
    Index64,
};

/// A chunk of 256 bits.
#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Bits256 {
    bits: [Bits64; 4],
}

impl Default for Bits256 {
    fn default() -> Self {
        Self {
            bits: Default::default(),
        }
    }
}

/// Iterator over the valid bits of a pack of 256 bits.
#[derive(Debug, Copy, Clone)]
pub struct Iter<'a> {
    bits: &'a Bits256,
    start: u16,
    end: u16,
}

impl<'a> Iter<'a> {
    fn new(bits256: &'a Bits256, len: u16) -> Self {
        Self {
            bits: bits256,
            start: 0,
            end: len,
        }
    }

    fn remaining(&self) -> u16 {
        self.end - self.start
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {}

impl<'a> Iterator for Iter<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        <Self as Iterator>::nth(self, 0)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        assert!(n < 256);
        let n = n as u16;
        if self.start + n >= self.end {
            return None
        }
        let start = self.start + n;
        self.start += 1 + n;
        Some(self.bits.get(start as u8))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining() as usize;
        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        self.remaining() as usize
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        <Self as DoubleEndedIterator>::nth_back(self, 0)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        assert!(n < 256);
        let n = n as u16;
        if self.start + n >= self.end {
            return None
        }
        self.end -= 1 + n;
        Some(self.bits.get(self.end as u8))
    }
}

/// Iterator over the valid mutable bits of a pack of 256 bits.
#[derive(Debug)]
pub struct IterMut<'a> {
    bits: &'a mut Bits256,
    start: u16,
    end: u16,
}

impl<'a> IterMut<'a> {
    fn new(bits256: &'a mut Bits256, len: u16) -> Self {
        Self {
            bits: bits256,
            start: 0,
            end: len,
        }
    }

    fn remaining(&self) -> u16 {
        self.end - self.start
    }

    /// Returns access for the given bit index with extended but valid lifetimes.
    fn get<'b>(&'b mut self, index: u8) -> BitRefMut<'a> {
        unsafe { BitRefMut::new(extend_lifetime(&mut self.bits), index) }
    }
}

impl<'a> ExactSizeIterator for IterMut<'a> {}

impl<'a> Iterator for IterMut<'a> {
    type Item = BitRefMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        <Self as Iterator>::nth(self, 0)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        assert!(n < 256);
        let n = n as u16;
        if self.start + n >= self.end {
            return None
        }
        let start = self.start + n;
        self.start += 1 + n;
        Some(self.get(start as u8))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining() as usize;
        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        self.remaining() as usize
    }
}

impl<'a> DoubleEndedIterator for IterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        <Self as DoubleEndedIterator>::nth_back(self, 0)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        assert!(n < 256);
        let n = n as u16;
        if self.start + n >= self.end {
            return None
        }
        self.end -= 1 + n;
        Some(self.get(self.end as u8))
    }
}

impl Bits256 {
    fn bits_at(&self, index: Index256) -> (&u64, Index64) {
        (&self.bits[(index / 64) as usize], index % 64)
    }

    fn bits_at_mut(&mut self, index: Index256) -> (&mut u64, Index64) {
        (&mut self.bits[(index / 64) as usize], index % 64)
    }

    /// Yields the first `len` bits of the pack of 256 bits.
    pub(super) fn iter(&self, len: u16) -> Iter {
        Iter::new(self, len)
    }

    /// Yields mutable accessors to the first `len` bits of the pack of 256 bits.
    pub(super) fn iter_mut(&mut self, len: u16) -> IterMut {
        IterMut::new(self, len)
    }

    /// Returns the bit value for the bit at the given index.
    pub fn get(&self, at: Index256) -> bool {
        let (bits64, pos64) = self.bits_at(at);
        bits64 & (0x01 << (63 - pos64)) != 0
    }

    /// Sets the bit value for the bit at the given index to the given value.
    pub(super) fn set_to(&mut self, at: Index256, new_value: bool) {
        if new_value {
            self.set(at)
        } else {
            self.reset(at)
        }
    }

    /// Flips the bit value for the bit at the given index.
    pub(super) fn flip(&mut self, at: Index256) {
        self.xor(at, true)
    }

    /// Sets the bit value for the bit at the given index to 1 (`true`).
    pub(super) fn set(&mut self, at: Index256) {
        self.or(at, true)
    }

    /// Sets the bit value for the bit at the given index to 0 (`false`).
    pub(super) fn reset(&mut self, at: Index256) {
        self.and(at, false)
    }

    fn op_at_with<F>(&mut self, at: Index256, rhs: bool, op: F)
    where
        F: FnOnce(&mut Bits64, Bits64),
    {
        let (bits64, pos64) = self.bits_at_mut(at);
        let rhs = (rhs as u64) << (63 - pos64);
        op(bits64, rhs);
    }

    /// Computes bitwise AND for the bit at the given index and `rhs`.
    pub(super) fn and(&mut self, at: Index256, rhs: bool) {
        self.op_at_with(at, !rhs, |bits64, rhs| *bits64 &= !rhs)
    }

    /// Computes bitwise OR for the bit at the given index and `rhs`.
    pub(super) fn or(&mut self, at: Index256, rhs: bool) {
        self.op_at_with(at, rhs, |bits64, rhs| *bits64 |= rhs)
    }

    /// Computes bitwise XOR for the bit at the given index and `rhs`.
    pub(super) fn xor(&mut self, at: Index256, rhs: bool) {
        self.op_at_with(at, rhs, |bits64, rhs| *bits64 ^= rhs)
    }

    /// Returns the position of the first zero bit if any.
    pub fn position_first_zero(&self) -> Option<u8> {
        let mut offset: u32 = 0;
        for bits64 in &self.bits {
            if *bits64 != !0 {
                return Some(((!bits64).leading_zeros() + offset) as u8)
            }
            offset += 64;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Bits256;

    #[test]
    fn default_works() {
        assert_eq!(
            Bits256::default(),
            Bits256 {
                bits: [0x00, 0x00, 0x00, 0x00],
            }
        );
    }

    fn populated_bits256() -> Bits256 {
        let mut bits256 = Bits256::default();
        for i in 0..256 {
            let i = i as u8;
            bits256.set_to(i, (i % 5) == 0 || (i % 13) == 0);
        }
        bits256
    }

    #[test]
    fn get_works() {
        let bits256 = populated_bits256();
        for i in 0..256 {
            let i = i as u8;
            assert_eq!(bits256.get(i), (i % 5) == 0 || (i % 13) == 0);
        }
    }

    #[test]
    fn set_works() {
        let mut bits256 = populated_bits256();
        for i in 0..256 {
            let i = i as u8;
            bits256.set(i);
            assert_eq!(bits256.get(i), true);
        }
    }

    #[test]
    fn reset_works() {
        let mut bits256 = populated_bits256();
        for i in 0..256 {
            let i = i as u8;
            bits256.reset(i);
            assert_eq!(bits256.get(i), false);
        }
    }

    #[test]
    fn and_works() {
        let mut bits256 = populated_bits256();
        for i in 0..256 {
            let i = i as u8;
            bits256.and(i, i % 2 == 0);
            assert_eq!(
                bits256.get(i),
                (i % 2) == 0 && ((i % 5) == 0 || (i % 13) == 0)
            );
        }
    }

    #[test]
    fn or_works() {
        let mut bits256 = populated_bits256();
        for i in 0..256 {
            let i = i as u8;
            bits256.or(i, i % 2 == 0);
            assert_eq!(
                bits256.get(i),
                (i % 2) == 0 || (i % 5) == 0 || (i % 13) == 0
            );
        }
    }

    #[test]
    fn xor_works() {
        let mut bits256 = populated_bits256();
        for i in 0..256 {
            let i = i as u8;
            bits256.xor(i, i % 2 == 0);
            let a = (i % 2) == 0;
            let b = (i % 5) == 0 || (i % 13) == 0;
            assert_eq!(bits256.get(i), a != b);
        }
    }

    #[test]
    fn position_first_zero_works() {
        // Zero bits256:
        let empty = Bits256::default();
        assert_eq!(empty.position_first_zero(), Some(0));
        // First bit is set:
        let first_bit_is_set = Bits256 {
            bits: [0x8000_0000_0000_0000, 0x00, 0x00, 0x00],
        };
        assert_eq!(first_bit_is_set.position_first_zero(), Some(1));
        // Last bit is unset:
        let first_bit_is_set = Bits256 {
            bits: [!0, !0, !0, !1],
        };
        assert_eq!(first_bit_is_set.position_first_zero(), Some(3 * 64 + 63));
        // Some middle bit is unset:
        let first_bit_is_set = Bits256 {
            bits: [!0, !0, !0xFFFF_FFFF, !1],
        };
        assert_eq!(first_bit_is_set.position_first_zero(), Some(2 * 64 + 32));
        // All bits set:
        let all_bits_set = Bits256 {
            bits: [!0, !0, !0, !0],
        };
        assert_eq!(all_bits_set.position_first_zero(), None);
    }
}
