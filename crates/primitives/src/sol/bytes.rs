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
    private::SolTypeValue,
    sol_data,
    SolType as AlloySolType,
};
use core::{
    borrow::Borrow,
    ops::Deref,
};
use ink_prelude::{
    borrow::Cow,
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
/// | `AsSolBytes<[u8; N]>` for `1 <= N <= 32` |  `bytesN` | e.g. `AsSolBytes<[u8; 1]>` <=> `bytes1` |
/// | `AsSolBytes<Vec<u8>>` |  `bytes` ||
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#fixed-size-byte-arrays>
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#bytes-and-string-as-arrays>
#[derive(Debug, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(TypeInfo))]
pub struct AsSolBytes<T: SolByteType>(pub T);

// Implement `SolTypeDecode` and `SolTypeEncode` for `AsBytes<T> where T: ByteType`.
impl<T: SolByteType> SolTypeDecode for AsSolBytes<T> {
    type AlloyType = T::AlloyType;

    fn detokenize(token: <Self::AlloyType as AlloySolType>::Token<'_>) -> Self {
        // Takes advantage of optimized `SolByteType::detokenize` implementations and
        // skips unnecessary conversions to `T::AlloyType::RustType`.
        Self(<T as SolByteType>::detokenize(token))
    }
}
impl<T: SolByteType> SolTypeEncode for AsSolBytes<T>
where
    AsSolBytes<T>: SolTypeValue<<T as SolByteType>::AlloyType>,
{
    type AlloyType = T::AlloyType;

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        <Self::AlloyType as AlloySolType>::tokenize(self)
    }
}
impl<T: SolByteType> crate::sol::types::private::Sealed for AsSolBytes<T> {}

// Implement `SolDecode` and `SolEncode` for `AsBytes<T> where T: ByteType`.
impl<T: SolByteType + Clone> SolDecode for AsSolBytes<T> {
    type SolType = AsSolBytes<T>;

    fn from_sol_type(value: Self::SolType) -> Self {
        value
    }
}
impl<T: SolByteType + Clone> SolEncode for AsSolBytes<T>
where
    AsSolBytes<T>: SolTypeValue<<T as SolByteType>::AlloyType>,
{
    type SolType = AsSolBytes<T>;

    fn to_sol_type(&self) -> Cow<Self::SolType> {
        Cow::Borrowed(self)
    }
}

// Implement core/standard traits for cheap representations as the inner type.
impl<T: SolByteType> Deref for AsSolBytes<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: SolByteType> Borrow<T> for AsSolBytes<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T: SolByteType> AsRef<T> for AsSolBytes<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl AsRef<[u8]> for AsSolBytes<Vec<u8>> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// A Rust equivalent of a Solidity ABI bytes type that implements logic for Solidity ABI
/// encoding/decoding.
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#fixed-size-byte-arrays>
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#bytes-and-string-as-arrays>
///
/// # Note
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
pub trait SolByteType: private::Sealed {
    /// Equivalent Solidity ABI bytes type from [`alloy_sol_types`].
    type AlloyType: AlloySolType;

    /// Detokenizes the byte type's value from the given token.
    fn detokenize(token: <Self::AlloyType as AlloySolType>::Token<'_>) -> Self;
}

// Implement `SolByteType` for `[u8; N]` and `Vec<u8>`.
impl<const N: usize> SolByteType for [u8; N]
where
    sol_data::ByteCount<N>: sol_data::SupportedFixedBytes,
{
    type AlloyType = sol_data::FixedBytes<N>;

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

impl SolByteType for Vec<u8> {
    type AlloyType = sol_data::Bytes;

    fn detokenize(token: <Self::AlloyType as AlloySolType>::Token<'_>) -> Self {
        // Converts token directly into `Vec<u8>`, skipping the conversion to
        // `alloy_sol_types::private::Bytes`, which then has to be converted back to
        // `Vec<u8>`.
        token.into_vec()
    }
}
impl private::Sealed for Vec<u8> {}

mod private {
    /// Seals the implementation of `SolByteType`.
    pub trait Sealed {}
}
