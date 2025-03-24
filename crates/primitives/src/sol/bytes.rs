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
    from::SolFrom,
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

/// A Rust equivalent of a Solidity ABI bytes type that implements logic for Solidity ABI
/// encoding/decoding.
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#fixed-size-byte-arrays>
///
/// Ref: <https://docs.soliditylang.org/en/latest/types.html#bytes-and-string-as-arrays>
///
/// # Note
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
pub trait SolByteType:
    SolTypeValue<Self::AlloyType>
    + SolFrom<<<Self as SolByteType>::AlloyType as AlloySolType>::RustType>
    + private::Sealed
{
    /// Equivalent Solidity ABI bytes type from [`alloy_sol_types`].
    type AlloyType: AlloySolType;

    /// Convert from [`alloy_sol_types::SolType::RustType`] (i.e. `alloy`'s bytes type) to
    /// `Self`.
    fn from_sol_type(value: <Self::AlloyType as AlloySolType>::RustType) -> Self;
}

// Implement `SolType` for `AsBytes<T> where T: ByteType`.
impl<T: SolByteType> SolTypeDecode for AsSolBytes<T>
where
    AsSolBytes<T>: SolTypeValue<<T as SolByteType>::AlloyType>,
{
    type AlloyType = T::AlloyType;
}
impl<T: SolByteType> SolTypeEncode for AsSolBytes<T>
where
    AsSolBytes<T>: SolTypeValue<<T as SolByteType>::AlloyType>,
{
    type AlloyType = T::AlloyType;
}
impl<T: SolByteType> crate::sol::types::private::Sealed for AsSolBytes<T> {}

// Implement `SolDecode` and `SolEncode` for `AsBytes<T> where T: ByteType`.
impl<T: SolByteType + Clone> SolDecode for AsSolBytes<T>
where
    AsSolBytes<T>: SolTypeValue<<T as SolByteType>::AlloyType>,
{
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

// Implement `SolFrom` for `AsBytes<T>`.
impl<T: SolByteType> SolFrom<<T::AlloyType as AlloySolType>::RustType> for AsSolBytes<T> {
    fn sol_from(value: <T::AlloyType as AlloySolType>::RustType) -> Self {
        Self(<T as SolByteType>::from_sol_type(value))
    }
}
impl<T: SolByteType> crate::sol::from::private::Sealed for AsSolBytes<T> {}

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

// Implement ByteType for `[u8; N]` and `Vec<u8>`.
impl<const N: usize> SolByteType for [u8; N]
where
    sol_data::ByteCount<N>: sol_data::SupportedFixedBytes,
{
    type AlloyType = sol_data::FixedBytes<N>;

    fn from_sol_type(value: <Self::AlloyType as AlloySolType>::RustType) -> Self {
        value.0
    }
}
impl<const N: usize> private::Sealed for [u8; N] {}

impl SolByteType for Vec<u8> {
    type AlloyType = sol_data::Bytes;

    fn from_sol_type(value: <Self::AlloyType as AlloySolType>::RustType) -> Self {
        value.0.to_vec()
    }
}
impl private::Sealed for Vec<u8> {}

mod private {
    /// Seals the implementation of `ByteType`.
    pub trait Sealed {}
}
