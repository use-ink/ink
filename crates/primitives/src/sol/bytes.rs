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

use core::{
    borrow::Borrow,
    ops::Deref,
};

use alloy_sol_types::{
    abi::token::{
        PackedSeqToken,
        WordToken,
    },
    sol_data,
    SolType as AlloySolType,
};
use ink_prelude::{
    boxed::Box,
    vec::Vec,
};
use scale::{
    Decode,
    Encode,
};
#[cfg(feature = "std")]
use scale_info::TypeInfo;

use crate::sol::{
    encodable::{
        DynSizeDefault,
        FixedSizeDefault,
    },
    types::SolTokenType,
    Error,
    SolDecode,
    SolEncode,
    SolTypeDecode,
    SolTypeEncode,
};

/// Newtype wrapper for Solidity ABI encoding/decoding `[u8; N]` for `1 <= N <= 32` as
/// fixed-size byte sequences.
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#fixed-size-byte-arrays>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(TypeInfo))]
#[repr(transparent)]
pub struct FixedBytes<const N: usize>(pub [u8; N]);

// Implements `SolTypeDecode` and `SolTypeEncode` for `FixedBytes<N>`.
impl<const N: usize> SolTypeDecode for FixedBytes<N>
where
    sol_data::ByteCount<N>: sol_data::SupportedFixedBytes,
{
    type AlloyType = sol_data::FixedBytes<N>;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, Error> {
        // Converts token directly into `[u8; N]`, skipping the conversion to
        // `alloy_sol_types::private::FixedBytes`, which would then has to be
        // unpacked to `[u8; N]`.
        // Ref: <https://github.com/alloy-rs/core/blob/5ae4fe0b246239602c97cc5a2f2e4bc780e2024a/crates/sol-types/src/types/data_type.rs#L204-L206>
        Ok(Self(
            token.0 .0[..N]
                .try_into()
                .expect("Expected a slice of N bytes"),
        ))
    }
}

impl<const N: usize> SolTypeEncode for FixedBytes<N>
where
    sol_data::ByteCount<N>: sol_data::SupportedFixedBytes,
{
    type AlloyType = sol_data::FixedBytes<N>;

    const DEFAULT_VALUE: Self::DefaultType = FixedSizeDefault::WORD;

    fn tokenize(&self) -> Self::TokenType<'_> {
        // Direct implementation simplifies generic implementations by removing
        // requirement for `SolTypeValue<Self::AlloyType>`.
        let mut word = [0; 32];
        word[..N].copy_from_slice(self.0.as_slice());
        WordToken::from(word)
    }
}

impl<const N: usize> SolTokenType for FixedBytes<N>
where
    sol_data::ByteCount<N>: sol_data::SupportedFixedBytes,
{
    type TokenType<'enc> = WordToken;

    type DefaultType = FixedSizeDefault;
}

impl<const N: usize> crate::sol::types::private::Sealed for FixedBytes<N> where
    sol_data::ByteCount<N>: sol_data::SupportedFixedBytes
{
}

// Implements `SolDecode` and `SolEncode` for `FixedBytes<N>`.
impl<const N: usize> SolDecode for FixedBytes<N>
where
    sol_data::ByteCount<N>: sol_data::SupportedFixedBytes,
{
    type SolType = FixedBytes<N>;

    fn from_sol_type(value: Self::SolType) -> Result<Self, Error> {
        Ok(value)
    }
}

impl<'a, const N: usize> SolEncode<'a> for FixedBytes<N>
where
    sol_data::ByteCount<N>: sol_data::SupportedFixedBytes,
{
    type SolType = &'a FixedBytes<N>;

    fn to_sol_type(&'a self) -> Self::SolType {
        self
    }
}

// Implements core/standard traits for cheap representations as the inner type.
impl<const N: usize> From<[u8; N]> for FixedBytes<N> {
    fn from(value: [u8; N]) -> Self {
        Self(value)
    }
}

impl From<u8> for FixedBytes<1> {
    fn from(value: u8) -> Self {
        Self([value; 1])
    }
}

impl<const N: usize> Deref for FixedBytes<N> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> Borrow<[u8; N]> for FixedBytes<N> {
    fn borrow(&self) -> &[u8; N] {
        &self.0
    }
}

impl<const N: usize> AsRef<[u8; N]> for FixedBytes<N> {
    fn as_ref(&self) -> &[u8; N] {
        &self.0
    }
}

/// Newtype wrapper for Solidity ABI encoding/decoding `Vec<u8>` as dynamic sized byte
/// sequences.
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#bytes-and-string-as-arrays>
#[derive(Debug, Clone, PartialEq, Eq, Default, Encode, Decode)]
#[cfg_attr(feature = "std", derive(TypeInfo))]
#[repr(transparent)]
pub struct DynBytes(pub Vec<u8>);

impl DynBytes {
    /// Constructs new empty `DynBytes` without allocating.
    pub const fn new() -> Self {
        Self(Vec::new())
    }
}

// Implements `SolTypeDecode` and `SolTypeEncode` for `DynBytes`.
impl SolTypeDecode for DynBytes {
    type AlloyType = sol_data::Bytes;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, Error> {
        // Converts token directly into `Vec<u8>`, skipping the conversion to
        // `alloy_sol_types::private::Bytes`, which then has to be converted back to
        // `Vec<u8>`.
        Ok(Self(token.into_vec()))
    }
}

impl SolTypeEncode for DynBytes {
    type AlloyType = sol_data::Bytes;

    const DEFAULT_VALUE: Self::DefaultType = DynSizeDefault;

    fn tokenize(&self) -> Self::TokenType<'_> {
        // Direct implementation simplifies generic implementations by removing
        // requirement for `SolTypeValue<Self::AlloyType>`.
        PackedSeqToken(self.0.as_slice())
    }
}

impl SolTokenType for DynBytes {
    type TokenType<'enc> = PackedSeqToken<'enc>;

    type DefaultType = DynSizeDefault;
}

impl crate::sol::types::private::Sealed for DynBytes {}

// Implements `SolDecode` and `SolEncode` for `DynBytes`.
impl SolDecode for DynBytes {
    type SolType = DynBytes;

    fn from_sol_type(value: Self::SolType) -> Result<Self, Error> {
        Ok(value)
    }
}

impl<'a> SolEncode<'a> for DynBytes {
    type SolType = &'a DynBytes;

    fn to_sol_type(&'a self) -> Self::SolType {
        self
    }
}

// Implements core/standard traits for cheap representations as the inner type.
impl From<Vec<u8>> for DynBytes {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<Box<[u8]>> for DynBytes {
    fn from(value: Box<[u8]>) -> Self {
        // Converts to `Vec<u8>` without clones or allocation.
        Self(value.into_vec())
    }
}

impl Deref for DynBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Borrow<[u8]> for DynBytes {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for DynBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
