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
/// | `SolBytes<u8>` |  `bytes1` ||
/// | `SolBytes<[u8; N]>` for `1 <= N <= 32` |  `bytesN` | e.g. `SolBytes<[u8; 32]>` <=> `bytes32` |
/// | `SolBytes<Vec<u8>>` | `bytes` ||
/// | `SolBytes<Box<[u8]>>` | `bytes` ||
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#fixed-size-byte-arrays>
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#bytes-and-string-as-arrays>
#[derive(Debug, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(TypeInfo))]
pub struct SolBytes<T: SolBytesType>(pub T);

// Implements `SolTypeDecode` and `SolTypeEncode` for `SolBytes<T>`.
impl<T: SolBytesType> SolTypeDecode for SolBytes<T> {
    type AlloyType = T::AlloyType;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, alloy_sol_types::Error> {
        // Takes advantage of optimized `SolBytesType::detokenize` implementations and
        // skips unnecessary conversions to `T::AlloyType::RustType`.
        Ok(Self(<T as SolBytesType>::detokenize(token)))
    }
}

impl<T: SolBytesType> SolTypeEncode for SolBytes<T> {
    type AlloyType = T::AlloyType;

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        <T as SolBytesType>::tokenize(self)
    }
}

impl<T: SolBytesType> crate::sol::types::private::Sealed for SolBytes<T> {}

// Implements `SolDecode` and `SolEncode` for `SolBytes<T>`.
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

// Implements core/standard traits for cheap representations as the inner type.
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

// Implements `SolBytesType` for `u8`, `[u8; N]`, `Vec<u8>` and `Box<[u8]>`.
impl SolBytesType for u8
where
    sol_data::ByteCount<1>: sol_data::SupportedFixedBytes,
{
    type AlloyType = sol_data::FixedBytes<1>;

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        // `u8` is encoded as `[u8; 1]` (i.e. `bytes1`).
        let mut word = [0; 32];
        word[0] = *self;
        WordToken::from(word)
    }

    fn detokenize(token: <Self::AlloyType as AlloySolType>::Token<'_>) -> Self {
        // `u8` is decoded as the first byte since `bytes1` is padded with trailing zeros.
        // Ref: <https://docs.soliditylang.org/en/latest/abi-spec.html#formal-specification-of-the-encoding>
        token.0 .0[0]
    }
}

impl private::Sealed for u8 {}

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

impl SolBytesType for Box<[u8]> {
    type AlloyType = sol_data::Bytes;

    fn detokenize(token: <Self::AlloyType as AlloySolType>::Token<'_>) -> Self {
        // Converts token directly into `Box<[u8]>`, skipping the conversion to
        // `alloy_sol_types::private::Bytes`, which then has to be converted back to
        // `Box<[u8]>`.
        Box::from(token.0)
    }

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        // Direct implementation simplifies generic implementations by removing
        // requirement for `SolValueType<Self::AlloyType>`.
        PackedSeqToken(self.as_ref())
    }
}

impl private::Sealed for Box<[u8]> {}

mod private {
    /// Seals the implementation of `SolBytesType`.
    pub trait Sealed {}
}
