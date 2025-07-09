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

use ink_prelude::vec::Vec;

use crate::sol::{
    Error,
    SolDecode,
    SolEncode,
    SolErrorDecode,
    SolErrorEncode,
};

impl<T, E> SolEncode<'_> for Result<T, E>
where
    T: for<'a> SolEncode<'a>,
    E: SolErrorEncode,
{
    // NOTE: Not actually used for encoding because of `encode` override below.
    type SolType = ();

    fn encode(&self) -> Vec<u8> {
        match self {
            Ok(val) => val.encode(),
            Err(err) => err.encode(),
        }
    }

    // NOTE: Not actually used for encoding because of `encode` override above.
    fn to_sol_type(&self) {}
}

/// Solidity ABI decode result data.
// Note: We define and implement this here (e.g. as opposed to an implementation
// in `ink_env`) because we need 2 generic implementations for `T: SolEncode` and
// `Result<T: SolEncode, E>` which `rustc` only allows if `Result<T, E>: !SolEncode`
// (i.e. `Result<T, E>` doesn't implement `SolEncode`). The latter negative condition is
// only allowed in this crate (because `SolEncode` is defined in this crate) as per Rust's
// coherence/orphan rules.
//
// Ref: <https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules>
pub trait SolResultDecode {
    /// Solidity ABI decode result data into this type.
    fn decode(data: &[u8], did_revert: bool) -> Result<Self, SolResultDecodeError>
    where
        Self: Sized;
}

/// Error representing reason for failing to decode Solidity ABI encoded result data.
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
    fn decode(data: &[u8], is_revert: bool) -> Result<Self, SolResultDecodeError>
    where
        Self: Sized,
    {
        if is_revert {
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
