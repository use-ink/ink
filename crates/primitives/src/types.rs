// Copyright (C) Parity Technologies (UK) Ltd.
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
use core::array::TryFromSliceError;
use derive_more::From;
use scale::{
    Decode,
    Encode,
};
#[cfg(feature = "std")]
use {
    scale_decode::DecodeAsType,
    scale_encode::EncodeAsType,
    scale_info::TypeInfo,
};

/// The default environment `AccountId` type.
///
/// # Note
///
/// This is a mirror of the `AccountId` type used in the default configuration
/// of PALLET contracts.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Decode, Encode, From,
)]
#[cfg_attr(feature = "std", derive(TypeInfo, DecodeAsType, EncodeAsType))]
pub struct AccountId([u8; 32]);

impl AsRef<[u8; 32]> for AccountId {
    #[inline]
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl AsMut<[u8; 32]> for AccountId {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }
}

impl AsRef<[u8]> for AccountId {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl AsMut<[u8]> for AccountId {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

impl<'a> TryFrom<&'a [u8]> for AccountId {
    type Error = TryFromSliceError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, TryFromSliceError> {
        let address = <[u8; 32]>::try_from(bytes)?;
        Ok(Self(address))
    }
}

/// The default environment `Hash` type.
///
/// # Note
///
/// This is a mirror of the `Hash` type used in the default configuration
/// of PALLET contracts.
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Hash,
    Decode,
    Encode,
    From,
    Default,
)]
#[cfg_attr(feature = "std", derive(TypeInfo, DecodeAsType, EncodeAsType))]
pub struct Hash([u8; 32]);

impl<'a> TryFrom<&'a [u8]> for Hash {
    type Error = TryFromSliceError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, TryFromSliceError> {
        let hash = <[u8; 32]>::try_from(bytes)?;
        Ok(Self(hash))
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl AsMut<[u8]> for Hash {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

impl From<Hash> for [u8; 32] {
    fn from(hash: Hash) -> Self {
        hash.0
    }
}

/// The equivalent of `Zero` for hashes.
///
/// A hash that consists only of 0 bits is clear.
pub trait Clear {
    /// The clear hash.
    const CLEAR_HASH: Self;

    /// Returns `true` if the hash is clear.
    fn is_clear(&self) -> bool;
}

impl Clear for [u8; 32] {
    const CLEAR_HASH: Self = [0x00; 32];

    fn is_clear(&self) -> bool {
        self == &Self::CLEAR_HASH
    }
}

impl Clear for Hash {
    const CLEAR_HASH: Self = Self(<[u8; 32] as Clear>::CLEAR_HASH);

    fn is_clear(&self) -> bool {
        <[u8; 32] as Clear>::is_clear(&self.0)
    }
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

/// A trait to enforce that a type should be an [`Environment::AccountId`].
///
/// If you have an [`Environment`] which uses an [`Environment::AccountId`] type other
/// than the ink! provided [`AccountId`](https://docs.rs/ink_primitives/latest/ink_primitives/struct.AccountId.html)
/// you will need to implement this trait for your [`Environment::AccountId`] concrete
/// type.
pub trait AccountIdGuard {}

/// The ink! provided [`AccountId`](https://docs.rs/ink_primitives/latest/ink_primitives/struct.AccountId.html)
/// used in the [`DefaultEnvironment`].
impl AccountIdGuard for AccountId {}

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub trait CodecAsType: scale_decode::DecodeAsType + scale_encode::EncodeAsType {}
        impl<T: scale_decode::DecodeAsType + scale_encode::EncodeAsType> CodecAsType for T {}
    } else {
        pub trait CodecAsType {}
        impl<T> CodecAsType for T {}
    }
}

/// The environmental types usable by contracts defined with ink!.
pub trait Environment: Clone {
    /// The maximum number of supported event topics provided by the runtime.
    ///
    /// The value must match the maximum number of supported event topics of the used
    /// runtime.
    const MAX_EVENT_TOPICS: usize;

    /// The account id type.
    type AccountId: 'static
        + scale::Codec
        + CodecAsType
        + Clone
        + PartialEq
        + Eq
        + Ord
        + AsRef<[u8]>
        + AsMut<[u8]>;

    /// The type of balances.
    type Balance: 'static
        + scale::Codec
        + CodecAsType
        + Copy
        + Clone
        + PartialEq
        + Eq
        + AtLeast32BitUnsigned
        + FromLittleEndian;

    /// The type of hash.
    type Hash: 'static
        + scale::Codec
        + CodecAsType
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
        + CodecAsType
        + Copy
        + Clone
        + PartialEq
        + Eq
        + AtLeast32BitUnsigned
        + FromLittleEndian;

    /// The type of block number.
    type BlockNumber: 'static
        + scale::Codec
        + CodecAsType
        + Copy
        + Clone
        + PartialEq
        + Eq
        + AtLeast32BitUnsigned
        + FromLittleEndian;

    /// The chain extension for the environment.
    ///
    /// This is a type that is defined through the `#[ink::chain_extension]` procedural
    /// macro. For more information about usage and definition click
    /// [this][chain_extension] link.
    ///
    /// [chain_extension]: https://paritytech.github.io/ink/ink/attr.chain_extension.html
    type ChainExtension;
}

/// Placeholder for chains that have no defined chain extension.
#[cfg_attr(feature = "std", derive(TypeInfo))]
pub enum NoChainExtension {}

/// The fundamental types of the default configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(TypeInfo))]
pub enum DefaultEnvironment {}

impl Environment for DefaultEnvironment {
    const MAX_EVENT_TOPICS: usize = 4;

    type AccountId = AccountId;
    type Balance = Balance;
    type Hash = Hash;
    type Timestamp = Timestamp;
    type BlockNumber = BlockNumber;
    type ChainExtension = NoChainExtension;
}

/// The default balance type.
pub type Balance = u128;

/// The default timestamp type.
pub type Timestamp = u64;

/// The default gas type.
pub type Gas = u64;

/// The default block number type.
pub type BlockNumber = u32;
