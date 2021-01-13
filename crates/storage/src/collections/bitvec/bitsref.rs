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

impl<'a> ChunkRef<&'a mut Bits256> {
    /// Creates a new exclusive 256-bit chunk access with the given length.
    pub(super) fn exclusive(bits: &'a mut Bits256, len: u32) -> Self {
        Self { bits, len }
    }

    /// Returns mutable access to a single bit if the index is out of bounds.
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

#[cfg(test)]
mod tests {
    use super::{
        Bits256,
        ChunkRef,
    };

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
    fn shared_works() {
        let len: u8 = 100;
        let bits = populated_bits256();
        let cref = ChunkRef::shared(&bits, len as u32);
        assert_eq!(cref.len(), len as u32);
        // Get works:
        for i in 0..len {
            assert_eq!(cref.get(i), Some(is_populated_bit_set(i)));
        }
        assert_eq!(cref.get(len), None);
        // Iter works:
        for (i, val) in cref.iter().enumerate() {
            assert_eq!(val, is_populated_bit_set(i as u8));
        }
    }

    #[test]
    fn exclusive_works() {
        let len: u8 = 100;
        let mut bits = populated_bits256();
        let mut cref = ChunkRef::exclusive(&mut bits, len as u32);
        assert_eq!(cref.len(), len as u32);
        // `get` and `get_mut` works:
        for i in 0..len {
            assert_eq!(cref.get(i), Some(is_populated_bit_set(i)));
            assert_eq!(
                cref.get_mut(i).map(|br| br.get()),
                Some(is_populated_bit_set(i))
            );
        }
        assert_eq!(cref.get(len), None);
        assert_eq!(cref.get_mut(len), None);
        // `iter` works:
        for (i, val) in cref.iter().enumerate() {
            assert_eq!(val, is_populated_bit_set(i as u8));
        }
    }

    #[test]
    fn position_first_zero_works() {
        let len = 256;
        let mut zeros = Default::default();
        let mut cref = ChunkRef::exclusive(&mut zeros, len);
        for i in 0..len {
            assert_eq!(cref.position_first_zero(), Some(i as u8));
            cref.get_mut(i as u8).unwrap().set();
        }
        // Now all bits are set to `1`:
        assert_eq!(cref.position_first_zero(), None);
    }

    #[test]
    fn iter_mut_works() {
        let len = 100;
        let mut zeros = Default::default();
        let mut cref = ChunkRef::exclusive(&mut zeros, len);
        // Initialize all bits with 0 and set them to 1 via `iter_mut`.
        // Then check if they are 1:
        for mut byte in cref.iter_mut() {
            assert!(!byte.get());
            byte.set();
        }
        assert!(cref.iter().all(|byte| byte));
    }
}
