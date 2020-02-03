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

#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

use super::*;
use ink_primitives::Key;

/// An allocator that is meant to allocate contract storage at
/// compile-time by simply bumping its current allocation key.
///
/// # Note
///
/// It is not designed to be used during contract execution and it
/// also cannot deallocate key allocated by it.
///
/// Users are recommended to use the [`DynAlloc`](struct.DynAlloc.html)
/// for dynamic storage allocation purposes instead.
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct BumpAlloc {
    /// The key offset used for all allocations.
    offset_key: Key,
}

impl BumpAlloc {
    /// Creates a new forward allocator for the given raw parts.
    ///
    /// # Safety
    ///
    /// Do not use this directly!
    /// This is meant to be used by pDSL internals only.
    #[inline(always)]
    pub unsafe fn from_raw_parts(offset_key: Key) -> Self {
        Self { offset_key }
    }

    /// Increase the forward alloc offset key by the given amount.
    fn inc_offset_key(&mut self, by: u64) {
        self.offset_key += by;
    }
}

impl Allocate for BumpAlloc {
    #[inline]
    fn alloc(&mut self, size: u64) -> Key {
        if size == 0 {
            panic!(
                "[psdl_core::BumpAlloc::alloc] Error: \
                 cannot allocate zero (0) bytes"
            )
        }
        let key = self.offset_key;
        self.inc_offset_key(size);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate() {
        let offset_key = Key([0x00; 32]);
        let mut bump_alloc = unsafe { BumpAlloc::from_raw_parts(offset_key) };
        assert_eq!(bump_alloc.alloc(1), offset_key);
        assert_eq!(bump_alloc.alloc(10), offset_key + 1_u32);
        assert_eq!(
            bump_alloc.alloc(u16::max_value() as u64),
            offset_key + 11_u32
        );
        assert_eq!(bump_alloc.alloc(2), offset_key + 0x1000A_u32);
        assert_eq!(bump_alloc.alloc(1), offset_key + 0x1000C_u32);
        assert_eq!(
            bump_alloc.alloc(u32::max_value() as u64),
            offset_key + 0x1000D_u32,
        );
        assert_eq!(
            bump_alloc.alloc(1),
            Key([
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x0C,
            ])
        )
    }

    #[test]
    #[should_panic]
    fn allocate_zero() {
        let offset_key = Key([0x00; 32]);
        let mut bump_alloc = unsafe { BumpAlloc::from_raw_parts(offset_key) };
        bump_alloc.alloc(0);
    }
}
