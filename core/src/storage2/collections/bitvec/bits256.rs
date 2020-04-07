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
    Bits64,
    Index256,
    Index64,
};
use crate::storage2::{
    pull_single_cell,
    PullAt,
    PushAt,
};
use ink_primitives::Key;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
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

impl Bits256 {
    fn bits_at(&self, index: Index256) -> (&u64, Index64) {
        (&self.bits[(index / 64) as usize], index % 64)
    }

    fn bits_at_mut(&mut self, index: Index256) -> (&mut u64, Index64) {
        (&mut self.bits[(index / 64) as usize], index % 64)
    }

    /// Returns the bit value for the bit at the given index.
    pub fn get(&self, at: Index256) -> bool {
        let (bits64, pos64) = self.bits_at(at);
        bits64 & (0x01 << (63 - pos64)) != 0
    }

    /// Sets the bit value for the bit at the given index to the given value.
    pub fn set_to(&mut self, at: Index256, new_value: bool) {
        if new_value {
            self.set(at)
        } else {
            self.reset(at)
        }
    }

    /// Flips the bit value for the bit at the given index.
    pub fn flip(&mut self, at: Index256) {
        self.xor(at, true)
    }

    /// Sets the bit value for the bit at the given index to 1 (`true`).
    pub fn set(&mut self, at: Index256) {
        self.or(at, true)
    }

    /// Sets the bit value for the bit at the given index to 0 (`false`).
    pub fn reset(&mut self, at: Index256) {
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
    pub fn and(&mut self, at: Index256, rhs: bool) {
        self.op_at_with(at, !rhs, |bits64, rhs| *bits64 &= !rhs)
    }

    /// Computes bitwise OR for the bit at the given index and `rhs`.
    pub fn or(&mut self, at: Index256, rhs: bool) {
        self.op_at_with(at, rhs, |bits64, rhs| *bits64 |= rhs)
    }

    /// Computes bitwise XOR for the bit at the given index and `rhs`.
    pub fn xor(&mut self, at: Index256, rhs: bool) {
        self.op_at_with(at, rhs, |bits64, rhs| *bits64 ^= rhs)
    }

    /// Returns the position of the first zero bit if any.
    pub fn position_first_zero(&self) -> Option<u8> {
        let mut offset = 0;
        for bits64 in &self.bits {
            if *bits64 != 0xFF {
                return Some(offset + (!bits64).leading_zeros() as u8)
            }
            offset += 64;
        }
        None
    }
}

impl PullAt for Bits256 {
    fn pull_at(at: Key) -> Self {
        pull_single_cell(at)
    }
}

impl PushAt for Bits256 {
    fn push_at(&self, at: Key) {
        crate::env::set_contract_storage(at, self)
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
}
