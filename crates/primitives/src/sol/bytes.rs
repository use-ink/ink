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
    SolType as AlloySolType,
    abi::token::{
        PackedSeqToken,
        WordToken,
    },
    sol_data,
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
    Error,
    SolDecode,
    SolEncode,
    SolTopicEncode,
    SolTypeDecode,
    SolTypeEncode,
    encodable::{
        DynSizeDefault,
        FixedSizeDefault,
    },
    types::SolTokenType,
    utils::{
        append_non_empty_member_topic_bytes,
        non_zero_multiple_of_32,
    },
};

/// Newtype wrapper for Solidity ABI encoding/decoding `[u8; N]` for `1 <= N <= 32` as
/// fixed-size byte sequences.
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#fixed-size-byte-arrays>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(TypeInfo))]
#[repr(transparent)]
pub struct FixedBytes<const N: usize>(pub [u8; N]);

impl<const N: usize> FixedBytes<N> {
    /// Converts a reference to `[u8; N]` into a reference to `FixedBytes<N>` (without
    /// copying).
    pub fn from_ref(value: &[u8; N]) -> &Self {
        // SAFETY: `FixedBytes<N>` is `#[repr(transparent)]` for `[u8; N]`,
        // so converting from `&[u8; N]` to `&FixedBytes<N>` is sound.
        unsafe { &*value.as_ptr().cast::<Self>() }
    }
}

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
            token.0.0[..N]
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

impl<const N: usize> SolTopicEncode for FixedBytes<N>
where
    sol_data::ByteCount<N>: sol_data::SupportedFixedBytes,
{
    fn encode_topic<H>(&self, _: H) -> [u8; 32]
    where
        H: Fn(&[u8], &mut [u8; 32]),
    {
        self.tokenize().0.0
    }

    fn topic_preimage(&self, buffer: &mut Vec<u8>) {
        buffer.extend(self.tokenize().0.0);
    }

    fn default_topic_preimage(buffer: &mut Vec<u8>) {
        buffer.extend([0u8; 32]);
    }

    fn topic_preimage_size(&self) -> usize {
        Self::default_topic_preimage_size()
    }

    fn default_topic_preimage_size() -> usize {
        32
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

/// Newtype wrapper for Solidity ABI encoding/decoding `Vec<u8>` as a dynamic sized byte
/// sequence.
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

impl DynBytes {
    /// Converts a reference to `Vec<u8>` into a reference to `DynBytes` (without
    /// copying).
    pub fn from_ref(value: &Vec<u8>) -> &Self {
        // SAFETY: `DynBytes` is `#[repr(transparent)]` for `Vec<u8>`,
        // so converting from `&Vec<u8>` to `&DynBytes` is sound.
        unsafe { &*(value as *const Vec<u8>).cast::<Self>() }
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

impl SolTopicEncode for DynBytes {
    fn encode_topic<H>(&self, hasher: H) -> [u8; 32]
    where
        H: Fn(&[u8], &mut [u8; 32]),
    {
        let mut output = [0u8; 32];
        hasher(self.0.as_slice(), &mut output);
        output
    }

    fn topic_preimage(&self, buffer: &mut Vec<u8>) {
        append_non_empty_member_topic_bytes(self.0.as_slice(), buffer);
    }

    fn default_topic_preimage(buffer: &mut Vec<u8>) {
        buffer.extend([0u8; 32]);
    }

    fn topic_preimage_size(&self) -> usize {
        non_zero_multiple_of_32(self.0.len())
    }

    fn default_topic_preimage_size() -> usize {
        32
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

/// Newtype wrapper for Solidity ABI encoding a byte slice (i.e. `&[u8]`) as a
/// dynamic sized byte sequence.
///
/// # Note
///
/// Only encoding is implemented for this type, see [`DynBytes`] for an equivalent "owned"
/// representation that supports both encoding and decoding.
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#bytes-and-string-as-arrays>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Encode)]
#[cfg_attr(feature = "std", derive(TypeInfo))]
#[repr(transparent)]
pub struct ByteSlice<'a>(pub &'a [u8]);

// Implements `SolTypeEncode` for `ByteSlice`.
impl SolTypeEncode for ByteSlice<'_> {
    type AlloyType = sol_data::Bytes;

    const DEFAULT_VALUE: Self::DefaultType = DynSizeDefault;

    fn tokenize(&self) -> Self::TokenType<'_> {
        // Direct implementation simplifies generic implementations by removing
        // requirement for `SolTypeValue<Self::AlloyType>`.
        PackedSeqToken(self.0)
    }
}

impl SolTopicEncode for ByteSlice<'_> {
    fn encode_topic<H>(&self, hasher: H) -> [u8; 32]
    where
        H: Fn(&[u8], &mut [u8; 32]),
    {
        let mut output = [0u8; 32];
        hasher(self.0, &mut output);
        output
    }

    fn topic_preimage(&self, buffer: &mut Vec<u8>) {
        append_non_empty_member_topic_bytes(self.0, buffer);
    }

    fn default_topic_preimage(buffer: &mut Vec<u8>) {
        buffer.extend([0u8; 32]);
    }

    fn topic_preimage_size(&self) -> usize {
        non_zero_multiple_of_32(self.0.len())
    }

    fn default_topic_preimage_size() -> usize {
        32
    }
}

impl SolTokenType for ByteSlice<'_> {
    type TokenType<'enc> = PackedSeqToken<'enc>;

    type DefaultType = DynSizeDefault;
}

impl crate::sol::types::private::Sealed for ByteSlice<'_> {}

// Implements `SolEncode` for `ByteSlice`.
impl<'a> SolEncode<'a> for ByteSlice<'a> {
    type SolType = &'a ByteSlice<'a>;

    fn to_sol_type(&'a self) -> Self::SolType {
        self
    }
}

// Implements core/standard traits for cheap representations as the inner type.
impl<'a> From<&'a [u8]> for ByteSlice<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self(value)
    }
}

impl Deref for ByteSlice<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Borrow<[u8]> for ByteSlice<'_> {
    fn borrow(&self) -> &[u8] {
        self.0
    }
}

impl AsRef<[u8]> for ByteSlice<'_> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}
