// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

use crate::arithmetic::AtLeast32BitUnsigned;
use ink_primitives::Clear;

/// The environmental types usable by contracts defined with ink!.
pub trait Environment {
    /// The maximum number of supported event topics provided by the runtime.
    ///
    /// The value must match the maximum number of supported event topics of the used runtime.
    const MAX_EVENT_TOPICS: usize;

    /// The address type.
    type AccountId: 'static
        + scale::Codec
        + Clone
        + PartialEq
        + Eq
        + Ord
        + AsRef<[u8]>
        + AsMut<[u8]>
        + Default;

    /// The type of balances.
    type Balance: 'static
        + scale::Codec
        + Copy
        + Clone
        + PartialEq
        + Eq
        + AtLeast32BitUnsigned
        + FromLittleEndian;

    /// The type of hash.
    type Hash: 'static
        + scale::Codec
        + Copy
        + Clone
        + Clear
        + PartialEq
        + Eq
        + Ord
        + AsRef<[u8]>
        + AsMut<[u8]>;

    /// The type of a timestamp.
    type Timestamp: 'static
        + scale::Codec
        + Copy
        + Clone
        + PartialEq
        + Eq
        + AtLeast32BitUnsigned
        + FromLittleEndian;

    /// The type of block number.
    type BlockNumber: 'static
        + scale::Codec
        + Copy
        + Clone
        + PartialEq
        + Eq
        + AtLeast32BitUnsigned
        + FromLittleEndian;

    /// The chain extension for the environment.
    ///
    /// This is a type that is defined through the `#[ink::chain_extension]` procedural macro.
    /// For more information about usage and definition click [this][chain_extension] link.
    ///
    /// [chain_extension]: https://paritytech.github.io/ink/ink/attr.chain_extension.html
    type ChainExtension;
}

/// Allows to instantiate a type from its little-endian bytes representation.
pub trait FromLittleEndian {
    /// The little-endian bytes representation.
    type Bytes: Default + AsRef<[u8]> + AsMut<[u8]>;

    /// Create a new instance from the little-endian bytes representation.
    fn from_le_bytes(bytes: Self::Bytes) -> Self;
}

impl FromLittleEndian for u8 {
    type Bytes = [u8; 1];

    #[inline]
    fn from_le_bytes(bytes: Self::Bytes) -> Self {
        u8::from_le_bytes(bytes)
    }
}

impl FromLittleEndian for u16 {
    type Bytes = [u8; 2];

    #[inline]
    fn from_le_bytes(bytes: Self::Bytes) -> Self {
        u16::from_le_bytes(bytes)
    }
}

impl FromLittleEndian for u32 {
    type Bytes = [u8; 4];

    #[inline]
    fn from_le_bytes(bytes: Self::Bytes) -> Self {
        u32::from_le_bytes(bytes)
    }
}

impl FromLittleEndian for u64 {
    type Bytes = [u8; 8];

    #[inline]
    fn from_le_bytes(bytes: Self::Bytes) -> Self {
        u64::from_le_bytes(bytes)
    }
}

impl FromLittleEndian for u128 {
    type Bytes = [u8; 16];

    #[inline]
    fn from_le_bytes(bytes: Self::Bytes) -> Self {
        u128::from_le_bytes(bytes)
    }
}
