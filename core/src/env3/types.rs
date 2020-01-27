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

//! Types for the default environment.
//!
//! These are simple mirrored types from the default SRML configuration.
//! Their interfaces and functionality might not be complete.
//!
//! Users are required to provide their own type definitions and `EnvTypes`
//! implementations in order to write ink! contracts for other chain configurations.

use crate::storage::Flush;
use core::{
    array::TryFromSliceError,
    convert::TryFrom,
};
use derive_more::From;
use ink_prelude::vec::Vec;
use scale::{
    Decode,
    Encode,
};
#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

use core::ops::{
    Add,
    AddAssign,
    Div,
    DivAssign,
    Mul,
    MulAssign,
    Sub,
    SubAssign,
};
use num_traits::{
    Bounded,
    One,
    Zero,
};

/// Types that allow for simple arithmetic operations.
///
/// Subset of all trait bounds copied over from what Substrate defines
/// for its `SimpleArithmetic` types. We can extend this in the future
/// if needed.
pub trait SimpleArithmetic:
    Sized
    + From<u32>
    + Bounded
    + Ord
    + PartialOrd<Self>
    + Zero
    + One
    + Bounded
    + Add
    + AddAssign
    + Sub
    + SubAssign
    + Mul
    + MulAssign
    + Div
    + DivAssign
{
}

impl<T> SimpleArithmetic for T where
    T: Sized
        + From<u32>
        + Bounded
        + Ord
        + PartialOrd<Self>
        + Zero
        + One
        + Bounded
        + Add
        + AddAssign
        + Sub
        + SubAssign
        + Mul
        + MulAssign
        + Div
        + DivAssign
{
}

/// The environmental types usable by contracts defined with ink!.
pub trait EnvTypes {
    /// The type of an address.
    type AccountId: 'static + scale::Codec + Clone + PartialEq + Eq + Ord;
    /// The type of balances.
    type Balance: 'static
        + scale::Codec
        + Copy
        + Clone
        + PartialEq
        + Eq
        + SimpleArithmetic;
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
    /// The type of timestamps.
    type Moment: 'static + scale::Codec + Copy + Clone + PartialEq + Eq + SimpleArithmetic;
    /// The type of block number.
    type BlockNumber: 'static
        + scale::Codec
        + Copy
        + Clone
        + PartialEq
        + Eq
        + SimpleArithmetic;
    /// The type of a call into the runtime
    type Call: 'static + scale::Codec;
}

/// Implemented by event types to communicate their topic hashes.
pub trait Topics<T>
where
    T: EnvTypes,
{
    /// Returns the topic hashes of `self`.
    fn topics(&self) -> &'static [<T as EnvTypes>::Hash];
}

/// The fundamental types of the default configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub enum DefaultEnvTypes {}

impl EnvTypes for DefaultEnvTypes {
    type AccountId = AccountId;
    type Balance = Balance;
    type Hash = Hash;
    type Moment = Moment;
    type BlockNumber = BlockNumber;
    type Call = Call;
}

/// The default balance type.
pub type Balance = u128;

/// The default moment type.
pub type Moment = u64;

/// The default block number type.
pub type BlockNumber = u64;

/// This call type guarantees to never be constructed.
///
/// This has the effect that users of the default SRML types are
/// not able to call back into the runtime.
/// This operation is generally unsupported because of the currently
/// implied additional overhead.
///
/// # Note
///
/// A user defined `Call` type is required for calling into the runtime.
/// For more info visit: https://github.com/paritytech/ink-types-node-runtime
#[derive(Debug)]
pub enum Call {}

impl Encode for Call {
    fn encode(&self) -> Vec<u8> {
        // The implementation enforces at runtime that `Encode` is not called
        // for the default SRML `Call` type but for performance reasons this check
        // is removed for the on-chain (release mode) version.
        debug_assert!(false, "cannot encode default SRML `Call` type");
        Vec::new()
    }
}

impl scale::Decode for Call {
    fn decode<I: scale::Input>(_value: &mut I) -> Result<Self, scale::Error> {
        // This implementation is only to satisfy the Decode constraint in the
        // test environment. Since Call cannot be constructed then just return
        // None, but this should never be called.
        Err("The default SRML `Call` type cannot be used for runtime calls".into())
    }
}

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
    Encode,
    Decode,
    From,
    Default,
)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct AccountId([u8; 32]);

impl<'a> TryFrom<&'a [u8]> for AccountId {
    type Error = TryFromSliceError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, TryFromSliceError> {
        let address = <[u8; 32]>::try_from(bytes)?;
        Ok(Self(address))
    }
}

impl Flush for AccountId {}

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
    Encode,
    Decode,
    From,
    Default,
)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct Hash([u8; 32]);

impl<'a> TryFrom<&'a [u8]> for Hash {
    type Error = TryFromSliceError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, TryFromSliceError> {
        let address = <[u8; 32]>::try_from(bytes)?;
        Ok(Self(address))
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

/// The equivalent of `Zero` for hashes.
///
/// A hash that consists only of 0 bits is clear.
pub trait Clear {
    /// Returns `true` if the hash is clear.
    fn is_clear(&self) -> bool;

    /// Returns a clear hash.
    fn clear() -> Self;
}

impl Clear for Hash {
    fn is_clear(&self) -> bool {
        self.as_ref().iter().all(|&byte| byte == 0x00)
    }

    fn clear() -> Self {
        Self([0x00; 32])
    }
}

impl Flush for Hash {}
