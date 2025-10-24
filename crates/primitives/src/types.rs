// Copyright (C) Use Ink (UK) Ltd.
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

use core::{
    array::TryFromSliceError,
    borrow::Borrow,
};
use derive_more::From;
use primitive_types::{
    H160,
    U256,
};
use scale::{
    Decode,
    Encode,
    MaxEncodedLen,
};
use sp_core::keccak_256;
#[cfg(feature = "std")]
use {
    scale_decode::DecodeAsType,
    scale_encode::EncodeAsType,
    scale_info::TypeInfo,
};

use crate::arithmetic::{
    AtLeast32BitUnsigned,
    Saturating,
};

/// The default environment `AccountId` type.
///
/// # Note
///
/// This is a mirror of the `AccountId` type used in the default configuration
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
    MaxEncodedLen,
    From,
)]
#[cfg_attr(feature = "std", derive(TypeInfo, DecodeAsType, EncodeAsType))]
pub struct AccountId(pub [u8; 32]);

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

impl Borrow<[u8; 32]> for AccountId {
    fn borrow(&self) -> &[u8; 32] {
        &self.0
    }
}

/// A Solidity compatible `address` type.
///
/// # Note
///
/// This is a type alias for the `H160` type used for addresses in `pallet-revive`.
// For rationale for using `H160` as the `address` type,
// see https://github.com/use-ink/ink/pull/2441#discussion_r2021230718.
pub type Address = H160;

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
    MaxEncodedLen,
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

impl Borrow<[u8; 32]> for Hash {
    fn borrow(&self) -> &[u8; 32] {
        &self.0
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

// todo
// impl Clear for H256 {
// const CLEAR_HASH: Self = H256::CLEAR_HASH;
//
// fn is_clear(&self) -> bool {
// self.as_bytes().iter().all(|&byte| byte == 0x00)
// }
// }

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

impl FromLittleEndian for U256 {
    type Bytes = [u8; 32];

    #[inline]
    fn from_le_bytes(bytes: Self::Bytes) -> Self {
        U256::from_little_endian(&bytes)
        //U256::from_le_bytes(bytes)
    }
}

/// todo remove
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

impl AccountIdGuard for Address {}

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
///
/// The types and consts in this trait must be the same as the chain to which
/// the contract is deployed to. We have a mechanism in `cargo-contract` that
/// attempts to check for type equality, but not everything can be compared.
pub trait Environment: Clone {
    /// The ratio between the decimal representation of the native `Balance` token
    /// and the ETH token.
    const NATIVE_TO_ETH_RATIO: u32;

    /// The account id type.
    type AccountId: 'static
        + scale::Codec
        + scale::MaxEncodedLen
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
        + Into<U256>
        + FromLittleEndian;

    /// The type of hash.
    type Hash: 'static
        + scale::Codec
        + scale::MaxEncodedLen
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

    /// TODO comment
    type EventRecord: 'static + scale::Codec;

    /// Converts from the generic `Balance` type to the Ethereum native `U256`.
    ///
    /// # Developer Note
    ///
    /// `pallet-revive` uses both types, hence we have to convert in between them
    /// for certain functions. Notice that precision loss might occur when converting
    /// the other way (from `U256` to `Balance`).
    ///
    /// See <https://github.com/paritytech/polkadot-sdk/pull/9101> for more details.
    fn native_to_eth(value: Self::Balance) -> U256 {
        value
            .saturating_mul(Self::NATIVE_TO_ETH_RATIO.into())
            .into()
    }
}

/// The fundamental types of the default configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(TypeInfo))]
pub enum DefaultEnvironment {}

impl Environment for DefaultEnvironment {
    // This number was chosen as it's also what `pallet-revive`
    // chooses by default. It's also the number present in the
    // `ink_sandbox` and the `ink-node`.
    const NATIVE_TO_ETH_RATIO: u32 = 100_000_000;

    type AccountId = AccountId;
    type Balance = Balance;
    type Hash = Hash;
    type Timestamp = Timestamp;
    type BlockNumber = BlockNumber;
    type EventRecord = EventRecord;
}

/// The default balance type.
pub type Balance = u128;

/// The default timestamp type.
pub type Timestamp = u64;

/// The default gas type.
pub type Gas = u64;

/// The default block number type.
pub type BlockNumber = u32;

// todo replace with ()
#[derive(Encode, Decode, MaxEncodedLen, Debug)]
pub struct RuntimeEvent();

/// The default event record type.
pub type EventRecord = EventRecordSpec<RuntimeEvent, Hash>;

#[derive(Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(TypeInfo))]
pub struct EventRecordSpec<E, H> {
    /// The phase of the block it happened in.
    pub phase: Phase,
    /// The event itself.
    pub event: E,
    /// The list of the topics this event has.
    pub topics: ink_prelude::vec::Vec<H>,
}

/// A phase of a block's execution.
#[derive(Debug, Encode, Decode, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(PartialEq, Eq, Clone, TypeInfo))]
pub enum Phase {
    /// Applying an extrinsic.
    ApplyExtrinsic(u32),
    /// Finalizing the block.
    Finalization,
    /// Initializing the block.
    Initialization,
}

/// The type of origins supported by `pallet-revive`.
#[derive(Clone, ::scale::Encode, ::scale::Decode, PartialEq)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
pub enum Origin<E: Environment> {
    Root,
    Signed(E::AccountId),
}

/// Copied from `pallet-revive`.
pub struct AccountIdMapper {}
impl AccountIdMapper {
    pub fn to_address(account_id: &[u8]) -> Address {
        let mut account_bytes: [u8; 32] = [0u8; 32];
        account_bytes.copy_from_slice(&account_id[..32]);
        if Self::is_eth_derived(account_id) {
            // this was originally an eth address
            // we just strip the 0xEE suffix to get the original address
            Address::from_slice(&account_bytes[..20])
        } else {
            // this is an (ed|sr)25510 derived address
            // avoid truncating the public key by hashing it first
            let account_hash = keccak_256(account_bytes.as_ref());
            Address::from_slice(&account_hash[12..])
        }
    }

    /// Returns true if the passed account id is controlled by an Ethereum key.
    ///
    /// This is a stateless check that just compares the last 12 bytes. Please note that
    /// it is theoretically possible to create an ed25519 keypair that passed this
    /// filter. However, this can't be used for an attack. It also won't happen by
    /// accident since everybody is using sr25519 where this is not a valid public key.
    fn is_eth_derived(account_bytes: &[u8]) -> bool {
        account_bytes[20..] == [0xEE; 12]
    }
}
