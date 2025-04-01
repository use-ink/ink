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

use alloy_sol_types::{
    abi::token::{
        PackedSeqToken,
        WordToken,
    },
    sol_data,
    SolType as AlloySolType,
};
use core::{
    borrow::Borrow,
    ops::Deref,
};
use ink_prelude::vec::Vec;
use scale::{
    Decode,
    Encode,
};
#[cfg(feature = "std")]
use scale_info::TypeInfo;

use crate::sol::{
    SolDecode,
    SolEncode,
    SolTypeDecode,
    SolTypeEncode,
};

/// Newtype wrapper for encoding/decoding `u8` sequences/collections as their equivalent
/// Solidity bytes representations.
///
/// | Rust/ink! type | Solidity ABI type | Notes |
/// | -------------- | ----------------- | ----- |
/// | `SolBytes<[u8; N]>` for `1 <= N <= 32` |  `bytesN` | e.g. `SolBytes<[u8; 1]>` <=> `bytes1` |
/// | `SolBytes<Vec<u8>>` |  `bytes` ||
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#fixed-size-byte-arrays>
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#bytes-and-string-as-arrays>
#[derive(Debug, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(TypeInfo))]
pub struct SolBytes<T: SolBytesType>(pub T);

// Implement `SolTypeDecode` and `SolTypeEncode` for `SolBytes<T>`.
impl<T: SolBytesType> SolTypeDecode for SolBytes<T> {
    type AlloyType = T::AlloyType;

    fn detokenize(token: <Self::AlloyType as AlloySolType>::Token<'_>) -> Self {
        // Takes advantage of optimized `SolBytesType::detokenize` implementations and
        // skips unnecessary conversions to `T::AlloyType::RustType`.
        Self(<T as SolBytesType>::detokenize(token))
    }
}

impl<T: SolBytesType> SolTypeEncode for SolBytes<T> {
    type AlloyType = T::AlloyType;

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        <T as SolBytesType>::tokenize(self)
    }
}

impl<T: SolBytesType> crate::sol::types::private::Sealed for SolBytes<T> {}

// Implement `SolDecode` and `SolEncode` for `SolBytes<T>`.
impl<T: SolBytesType> SolDecode for SolBytes<T> {
    type SolType = SolBytes<T>;

    fn from_sol_type(value: Self::SolType) -> Self {
        value
    }
}

impl<'a, T: SolBytesType + 'a> SolEncode<'a> for SolBytes<T> {
    type SolType = &'a SolBytes<T>;

    fn to_sol_type(&'a self) -> Self::SolType {
        self
    }
}

// Implement core/standard traits for cheap representations as the inner type.
impl<T: SolBytesType> Deref for SolBytes<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: SolBytesType> Borrow<T> for SolBytes<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T: SolBytesType> AsRef<T> for SolBytes<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl AsRef<[u8]> for SolBytes<Vec<u8>> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// A Rust/ink! equivalent of a Solidity ABI bytes type that implements logic for Solidity
/// ABI encoding/decoding.
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#fixed-size-byte-arrays>
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#bytes-and-string-as-arrays>
///
/// # Note
///
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
pub trait SolBytesType: private::Sealed {
    /// Equivalent Solidity ABI bytes type from [`alloy_sol_types`].
    type AlloyType: AlloySolType;

    /// Tokenizes the given value into a [`Self::AlloyType`] token.
    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_>;

    /// Detokenizes the byte type's value from the given token.
    fn detokenize(token: <Self::AlloyType as AlloySolType>::Token<'_>) -> Self;
}

// Implement `SolBytesType` for `[u8; N]` and `Vec<u8>`.
impl<const N: usize> SolBytesType for [u8; N]
where
    sol_data::ByteCount<N>: sol_data::SupportedFixedBytes,
{
    type AlloyType = sol_data::FixedBytes<N>;

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        // Direct implementation simplifies generic implementations by removing
        // requirement for `SolValueType<Self::AlloyType>`.
        let mut word = [0; 32];
        word[..N].copy_from_slice(self.as_slice());
        WordToken::from(word)
    }

    fn detokenize(token: <Self::AlloyType as AlloySolType>::Token<'_>) -> Self {
        // Converts token directly into `[u8; N]`, skipping the conversion to
        // `alloy_sol_types::private::FixedBytes`, which then has to be unpacked to
        // `[u8; N]`.
        // Ref: <https://github.com/alloy-rs/core/blob/5ae4fe0b246239602c97cc5a2f2e4bc780e2024a/crates/sol-types/src/types/data_type.rs#L204-L206>
        token.0 .0[..N]
            .try_into()
            .expect("Expected a slice of N bytes")
    }
}

impl<const N: usize> private::Sealed for [u8; N] {}

impl SolBytesType for Vec<u8> {
    type AlloyType = sol_data::Bytes;

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        // Direct implementation simplifies generic implementations by removing
        // requirement for `SolValueType<Self::AlloyType>`.
        PackedSeqToken(self.as_slice())
    }

    fn detokenize(token: <Self::AlloyType as AlloySolType>::Token<'_>) -> Self {
        // Converts token directly into `Vec<u8>`, skipping the conversion to
        // `alloy_sol_types::private::Bytes`, which then has to be converted back to
        // `Vec<u8>`.
        token.into_vec()
    }
}

impl private::Sealed for Vec<u8> {}

mod private {
    /// Seals the implementation of `SolBytesType`.
    pub trait Sealed {}
}
