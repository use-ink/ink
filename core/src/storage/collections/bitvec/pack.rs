// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use scale::{
    Decode,
    Encode,
};
#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

/// The underlying representation type for a pack.
pub type BitPackRepr = u32;

/// A pack of 32 bits.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
#[repr(transparent)]
pub struct BitPack {
    /// The actual bits.
    bits: BitPackRepr,
}

/// Error indicating an invalid bit pack index.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct InvalidBitPackIndex;

/// Result type when working with bit packs.
type BitPackResult<T> = Result<T, InvalidBitPackIndex>;

impl BitPack {
    /// The number of bits of a bit pack.
    pub const BITS: u32 = (core::mem::size_of::<BitPackRepr>() * 8) as u32;

    /// Creates a new bit pack from the given underlying representation.
    pub const fn new(bits: BitPackRepr) -> Self {
        Self { bits }
    }

    /// Checks if `n` is within bounds of a bit pack and returns it if so.
    fn validate_index(n: u32) -> BitPackResult<u32> {
        if n >= Self::BITS {
            return Err(InvalidBitPackIndex)
        }
        Ok(n)
    }

    /// Returns the value of the n-th bit.
    pub fn get(self, n: u32) -> bool {
        Self::validate_index(n)
            .map(|n| (self.bits & (0x1 << (Self::BITS - n - 1))) != 0)
            .unwrap()
    }

    /// Sets the value of the n-th bit.
    pub fn set(&mut self, n: u32, value: bool) {
        Self::validate_index(n)
            .map(|n| {
                if value {
                    self.bits |= 0x1 << (Self::BITS - n - 1);
                } else {
                    self.bits &= !(0x1 << (Self::BITS - n - 1));
                }
            })
            .unwrap()
    }

    /// Flips the value of the n-th bit.
    pub fn flip(&mut self, n: u32) {
        Self::validate_index(n)
            .map(|n| {
                self.bits ^= 0x1 << (Self::BITS - n - 1);
            })
            .unwrap()
    }

    /// Returns the position of the first set bit if any.
    pub fn first_set_position(self) -> Option<u32> {
        if self.bits == 0x0 {
            return None
        }
        Some(self.bits.leading_zeros())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_index() {
        assert_eq!(BitPack::validate_index(0), Ok(0));
        assert_eq!(BitPack::validate_index(1), Ok(1));
        assert_eq!(
            BitPack::validate_index(BitPack::BITS - 1),
            Ok(BitPack::BITS - 1)
        );
        assert_eq!(
            BitPack::validate_index(BitPack::BITS),
            Err(InvalidBitPackIndex)
        );
    }

    #[test]
    fn get() {
        let bp = BitPack::new(0x0001_0000); // 15th bit set
        for n in 0_u32..BitPack::BITS {
            assert_eq!(bp.get(n), n == 15)
        }
    }

    #[test]
    #[should_panic]
    fn get_out_of_bounds() {
        BitPack::new(0x0).get(32);
    }

    #[test]
    fn set() {
        let mut bp = BitPack::new(0x0);
        bp.set(15, true);
        assert_eq!(bp, BitPack::new(0x0001_0000));
        bp.set(15, false);
        assert_eq!(bp, BitPack::new(0x0));
    }

    #[test]
    #[should_panic]
    fn set_out_of_bounds() {
        BitPack::new(0x0).set(32, true);
    }

    #[test]
    fn flip() {
        let mut bp = BitPack::new(0x0);
        bp.flip(15);
        assert_eq!(bp, BitPack::new(0x0001_0000));
        bp.flip(15);
        assert_eq!(bp, BitPack::new(0x0));
    }

    #[test]
    #[should_panic]
    fn flip_out_of_bounds() {
        BitPack::new(0x0).flip(32)
    }

    #[test]
    fn first_set_position() {
        assert_eq!(BitPack::new(0x0).first_set_position(), None);
        assert_eq!(BitPack::new(0x0001_0000).first_set_position(), Some(15));
        assert_eq!(BitPack::new(0xFFFF_FFFF).first_set_position(), Some(0));
    }
}
