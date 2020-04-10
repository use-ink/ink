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

use crate::{
    hash::{
        Blake2x256,
        Wrap,
    },
    storage2::{
        pull_single_cell,
        KeyPtr,
        PullAt,
        PullForward,
        PushAt,
        PushForward,
        ClearForward,
        ClearAt,
        StorageFootprint,
    },
};
use ink_primitives::Key;

/// A unique dynamic allocation.
///
/// This can refer to a dynamically allocated storage cell.
/// It has been created by a dynamic storage allocator.
/// The initiater of the allocation has to make sure to deallocate
/// this dynamic allocation again using the same dynamic allocator
/// if it is no longer in use.
///
/// # Note
///
/// Normally instances of this type are not used directly and instead
/// a [`storage::Box`](`crate::storage2::Box`) is used instead.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, scale::Encode, scale::Decode,
)]
pub struct DynamicAllocation(pub(super) u32);

impl DynamicAllocation {
    /// Returns the allocation identifier as `u32`.
    pub(super) fn get(&self) -> u32 {
        self.0
    }

    /// Returns the storage key associated with this dynamic allocation.
    pub fn key(&self) -> Key {
        // We create a 25-bytes buffer for the hashing.
        // This is due to the fact that we prepend the `u32` encoded identifier
        // with the `b"DYNAMICALLY ALLOCATED"` byte string which has a length
        // 21 bytes. Since `u32` always has an encoding length of 4 bytes we
        // end up requiring 25 bytes in total.
        // Optimization Opportunity:
        // Since ink! always runs single threaded we could make this buffer
        // static and instead reuse its contents with every invocation of this
        // method. However, this would introduce `unsafe` Rust usage.
        #[rustfmt::skip]
        let mut buffer: [u8; 25] = [
            b'D', b'Y', b'N', b'A', b'M', b'I', b'C', b'A', b'L', b'L', b'Y',
            b' ',
            b'A', b'L', b'L', b'O', b'C', b'A', b'T', b'E', b'D',
            b'_', b'_', b'_', b'_',
        ];
        // Encode the `u32` identifier requires a 4 bytes buffer.
        let mut hash_buffer = Wrap::from(&mut buffer[21..25]);
        <u32 as scale::Encode>::encode_to(&self.0, &mut hash_buffer);
        let mut output = [0x00_u8; 32];
        <Blake2x256>::hash_bytes_using(&buffer, &mut output);
        Key::from(output)
    }
}

impl StorageFootprint for DynamicAllocation {
    const VALUE: u64 = 1;
}

impl PullForward for DynamicAllocation {
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        <Self as PullAt>::pull_at(ptr.next_for::<Self>())
    }
}

impl PullAt for DynamicAllocation {
    fn pull_at(at: Key) -> Self {
        pull_single_cell(at)
    }
}

impl PushForward for DynamicAllocation {
    fn push_forward(&self, ptr: &mut KeyPtr) {
        <Self as PushAt>::push_at(self, ptr.next_for::<Self>())
    }
}

impl PushAt for DynamicAllocation {
    fn push_at(&self, at: Key) {
        crate::env::set_contract_storage(at, self)
    }
}

impl ClearForward for DynamicAllocation {
    fn clear_forward(&self, ptr: &mut KeyPtr) {}
}

impl ClearAt for DynamicAllocation {
    fn clear_at(&self, at: Key) {}
}
