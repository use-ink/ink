// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

use core::convert::TryFrom;
use core::array::TryFromSliceError;

use crate::env::EnvTypes;
use parity_codec::{
    Decode,
    Encode,
};

/// The SRML fundamental types.
pub struct DefaultSrmlTypes;

impl EnvTypes for DefaultSrmlTypes {
    type Address = self::Address;
    type Balance = self::Balance;
    type Hash = self::Hash;
}

/// The default SRML address type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct Address([u8; 32]);

impl Address {
    pub fn new(address: [u8; 32]) -> Address {
        Address(address)
    }
}

impl<'a> TryFrom<&'a [u8]> for Address {
    type Error = TryFromSliceError;

    fn try_from(bytes: &'a [u8]) -> Result<Address, TryFromSliceError> {
        let address = <[u8; 32]>::try_from(bytes)?;
        Ok(Address(address))
    }
}

/// The default SRML balance type.
pub type Balance = u64;

/// The default SRML hash type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn new(hash: [u8; 32]) -> Hash {
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