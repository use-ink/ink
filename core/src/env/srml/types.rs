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

use core::{
    array::TryFromSliceError,
    convert::TryFrom,
};

use scale::{
    Decode,
    Encode,
};

use crate::{
    env::EnvTypes,
    storage::Flush,
};

#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

/// The SRML fundamental types.
#[cfg_attr(feature = "test-env", derive(Debug, Clone, PartialEq, Eq))]
pub enum DefaultSrmlTypes {}

/// Empty enum for default Call type, so it cannot be constructed.
/// For calling into the runtime, a user defined Call type required.
/// See https://github.com/paritytech/ink-types-node-runtime.
///
/// # Note
///
/// Some traits are only implemented to satisfy the constraints of the test
/// environment, in order to keep the code size small.
#[cfg_attr(feature = "test-env", derive(Debug, Clone, PartialEq, Eq))]
pub enum Call {}
impl scale::Encode for Call {}

/// This implementation is only to satisfy the Decode constraint in the
/// test environment. Since Call cannot be constructed then just return
/// None, but this should never be called.
#[cfg(feature = "test-env")]
impl scale::Decode for Call {
    fn decode<I: scale::Input>(_value: &mut I) -> Result<Self, scale::Error> {
        Err("Call cannot be instantiated".into())
    }
}

impl EnvTypes for DefaultSrmlTypes {
    type AccountId = AccountId;
    type Balance = Balance;
    type Hash = Hash;
    type Moment = Moment;
    type BlockNumber = BlockNumber;
    type Call = Call;
}

/// The default SRML address type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct AccountId([u8; 32]);

impl From<[u8; 32]> for AccountId {
    fn from(address: [u8; 32]) -> AccountId {
        AccountId(address)
    }
}

impl<'a> TryFrom<&'a [u8]> for AccountId {
    type Error = TryFromSliceError;

    fn try_from(bytes: &'a [u8]) -> Result<AccountId, TryFromSliceError> {
        let address = <[u8; 32]>::try_from(bytes)?;
        Ok(AccountId(address))
    }
}

/// The default SRML balance type.
pub type Balance = u128;

/// The default SRML hash type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct Hash([u8; 32]);

impl Default for Hash {
    fn default() -> Self {
        Hash(Default::default())
    }
}

impl From<[u8; 32]> for Hash {
    fn from(hash: [u8; 32]) -> Hash {
        Hash(hash)
    }
}

impl<'a> TryFrom<&'a [u8]> for Hash {
    type Error = TryFromSliceError;

    fn try_from(bytes: &'a [u8]) -> Result<Hash, TryFromSliceError> {
        let hash = <[u8; 32]>::try_from(bytes)?;
        Ok(Hash(hash))
    }
}

/// The default SRML moment type.
pub type Moment = u64;

/// The default SRML blocknumber type.
pub type BlockNumber = u64;

impl Flush for AccountId {
    fn flush(&mut self) {}
}

impl Flush for Hash {
    fn flush(&mut self) {}
}
