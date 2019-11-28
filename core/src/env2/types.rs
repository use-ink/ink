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

//! Types for the default SRML environment.
//!
//! These are simple mirrored types from the default SRML configuration.
//! Their interfaces and functionality might not be complete.
//!
//! Users are required to provide their own type definitions and `EnvTypes`
//! implementations in order to write ink! contracts for other chain configurations.

use core::{
    array::TryFromSliceError,
    convert::TryFrom,
};

use derive_more::From;
use scale::{
    Decode,
    Encode,
};
#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

use crate::{
    env2::EnvTypes,
    storage::Flush,
};
use ink_prelude::vec::Vec;

/// The fundamental types of the SRML default configuration.
#[cfg_attr(feature = "test-env", derive(Debug, Clone, PartialEq, Eq))]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
pub enum DefaultSrmlTypes {}

impl EnvTypes for DefaultSrmlTypes {
    type AccountId = AccountId;
    type Balance = Balance;
    type Hash = Hash;
    type Moment = Moment;
    type BlockNumber = BlockNumber;
    type Call = Call;
}

/// The default SRML balance type.
pub type Balance = u128;

/// The default SRML moment type.
pub type Moment = u64;

/// The default SRML blocknumber type.
pub type BlockNumber = u64;

/// Empty enum for default Call type, so it cannot be constructed.
/// For calling into the runtime, a user defined Call type required.
/// See https://github.com/paritytech/ink-types-node-runtime.
///
/// # Note
///
/// Some traits are only implemented to satisfy the constraints of the test
/// environment, in order to keep the code size small.

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
#[cfg_attr(feature = "test-env", derive(Debug, Clone, PartialEq, Eq))]
pub enum Call {}

/// The implementation enforces at runtime that `Encode` is not called
/// for the default SRML `Call` type but for performance reasons this check
/// is removed for the on-chain (release mode) version.
impl Encode for Call {
    fn encode(&self) -> Vec<u8> {
        debug_assert!(false, "cannot encode default SRML `Call` type");
        Vec::new()
    }
}

/// This implementation is only to satisfy the Decode constraint in the
/// test environment. Since Call cannot be constructed then just return
/// None, but this should never be called.
#[cfg(feature = "test-env")]
impl scale::Decode for Call {
    fn decode<I: scale::Input>(_value: &mut I) -> Result<Self, scale::Error> {
        Err("The default SRML `Call` type cannot be used for runtime calls".into())
    }
}

/// The default SRML `AccountId` type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode, From, Default)]
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

/// The default SRML `Hash` type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode, From, Default)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct Hash([u8; 32]);

impl<'a> TryFrom<&'a [u8]> for Hash {
    type Error = TryFromSliceError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, TryFromSliceError> {
        let address = <[u8; 32]>::try_from(bytes)?;
        Ok(Self(address))
    }
}

impl Flush for Hash {}
