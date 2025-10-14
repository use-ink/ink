// Copyright (C) ink! contributors.
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

use alloy_sol_types::SolType as AlloySolType;
use ink_prelude::vec::Vec;

use crate::sol::{
    Error,
    SolDecode,
    SolEncode,
    SolErrorDecode,
    SolErrorEncode,
    SolTypeDecode,
    SolTypeEncode,
};

/// Solidity ABI encode return data.
pub trait SolResultEncode<'a> {
    /// Equivalent Solidity ABI type representation.
    type SolType: SolTypeEncode;

    /// Name of equivalent Solidity ABI type.
    const SOL_NAME: &'static str =
        <<Self::SolType as SolTypeEncode>::AlloyType as AlloySolType>::SOL_NAME;

    /// Solidity ABI encode this type as return data.
    fn encode(&'a self) -> Vec<u8>;
}

impl<'a, T> SolResultEncode<'a> for T
where
    T: SolEncode<'a>,
{
    type SolType = <T as SolEncode<'a>>::SolType;

    fn encode(&'a self) -> Vec<u8> {
        T::encode(self)
    }
}

impl<'a, T, E> SolResultEncode<'a> for Result<T, E>
where
    T: SolEncode<'a>,
    E: SolErrorEncode,
{
    type SolType = <T as SolEncode<'a>>::SolType;

    fn encode(&'a self) -> Vec<u8> {
        match self {
            Ok(val) => val.encode(),
            Err(err) => err.encode(),
        }
    }
}

/// Solidity ABI decode return data.
// Note: We define and implement this here (e.g. as opposed to an implementation
// in `ink_env`) because we need 2 generic implementations for `T: SolEncode` and
// `Result<T: SolEncode, E>` which Rust only allows if `Result<T, E>: !SolEncode`
// (i.e. `Result<T, E>` doesn't implement `SolEncode`). The latter negative condition is
// only allowed in this crate (because `SolEncode` is defined in this crate) as per Rust's
// coherence/orphan rules.
//
// Ref: <https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules>
pub trait SolResultDecode {
    /// Equivalent Solidity ABI type representation.
    type SolType: SolTypeDecode;

    /// Name of equivalent Solidity ABI type.
    const SOL_NAME: &'static str =
        <<Self::SolType as SolTypeDecode>::AlloyType as AlloySolType>::SOL_NAME;

    /// Solidity ABI decode return data into this type.
    fn decode(data: &[u8], did_revert: bool) -> Result<Self, SolResultDecodeError>
    where
        Self: Sized;
}

/// Error representing reason for failing to decode Solidity ABI encoded return data.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SolResultDecodeError {
    /// Tried to decode revert/error data into a non-Result type.
    NonResultFromRevert,
    /// A general decoding error.
    Decode,
}

impl From<Error> for SolResultDecodeError {
    fn from(_: Error) -> Self {
        SolResultDecodeError::Decode
    }
}

impl<T> SolResultDecode for T
where
    T: SolDecode,
{
    type SolType = <T as SolDecode>::SolType;

    fn decode(data: &[u8], did_revert: bool) -> Result<Self, SolResultDecodeError>
    where
        Self: Sized,
    {
        if did_revert {
            Err(SolResultDecodeError::NonResultFromRevert)
        } else {
            Ok(T::decode(data)?)
        }
    }
}

impl<T, E> SolResultDecode for Result<T, E>
where
    T: SolDecode,
    E: SolErrorDecode,
{
    type SolType = <T as SolDecode>::SolType;

    fn decode(data: &[u8], did_revert: bool) -> Result<Self, SolResultDecodeError>
    where
        Self: Sized,
    {
        if did_revert {
            Ok(E::decode(data).map(|err| Err(err))?)
        } else {
            Ok(T::decode(data).map(|val| Ok(val))?)
        }
    }
}
