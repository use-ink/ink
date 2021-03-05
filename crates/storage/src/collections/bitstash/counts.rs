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

use core::ops::{
    Index,
    IndexMut,
};

/// Stores the number of set bits for each 256-bits block in a compact `u8`.
#[derive(Debug, Default, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct CountFree {
    /// Set bits per 256-bit chunk.
    counts: [u8; 32],
    /// Since with `u8` can only count up to 255 but there might be the need
    /// to count up to 256 bits for 256-bit chunks we need to store one extra
    /// bit per counter to determine filled chunks.
    full: FullMask,
}

impl Index<u8> for CountFree {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        &self.counts[index as usize]
    }
}

impl IndexMut<u8> for CountFree {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.counts[index as usize]
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct FullMask(u32);

impl FullMask {
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
    /// Returns the position of the first free `u8` in the free counts.
    ///
    /// Returns `None` if all counts are `0xFF`.
    pub fn position_first_zero(&self) -> Option<u8> {
        let i = (!self.full.0).leading_zeros();
        if i == 32 {
            return None
        }
        Some(i as u8)
    }

    /// Increases the number of set bits for the given index.
    ///
    /// # Panics
    ///
    /// - If the given index is out of bounds.
    /// - If the increment would cause an overflow.
    pub fn inc(&mut self, index: usize) {
        assert!(index < 32, "index is out of bounds");
        if self.counts[index] == !0 {
            self.full.set_full(index as u8);
        } else {
            self.counts[index] += 1;
        }
    }

    /// Decreases the number of set bits for the given index.
    ///
    /// Returns the new number of set bits.
    ///
    /// # Panics
    ///
    /// - If the given index is out of bounds.
    /// - If the decrement would cause an overflow.
    pub fn dec(&mut self, index: u8) -> u8 {
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
