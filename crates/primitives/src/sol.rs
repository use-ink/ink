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

//! Abstractions for implementing Solidity ABI encoding/decoding for arbitrary Rust types.

mod bytes;
mod from;
mod types;

#[cfg(test)]
mod tests;

use alloy_sol_types::{
    sol_data,
    SolType as AlloySolType,
};
use core::ops::Deref;
use impl_trait_for_tuples::impl_for_tuples;
use ink_prelude::{
    borrow::Cow,
    vec::Vec,
};

pub use bytes::AsSolBytes;
pub use types::{
    SolTypeDecode,
    SolTypeEncode,
};

use crate::types::{
    AccountId,
    Hash,
};

/// Maps an arbitrary Rust/ink! type to a Solidity ABI type equivalent for Solidity
/// ABI decoding.
///
/// # Note
/// Implementing this trait entails:
/// - Declaring the equivalent Solidity ABI type via the `SolType` associated type. See
///   the [docs for sealed `SolTypeDecode` trait][SolTypeDecode] for a table of Rust/ink!
///   primitive types mapped to their equivalent Solidity ABI type.
/// - Implementing the `from_sol_type` method which defines how to convert from the
///   Solidity ABI representation (i.e. `Self::SolType`) to this type.
///
/// # Example
///
/// ```
/// use ink_primitives::SolDecode;
///
/// // Example arbitrary type.
/// struct MyType {
///     size: u8,
///     status: bool,
/// }
///
/// // `SolDecode` implementation/mapping.
/// impl SolDecode for MyType {
///     type SolType = (u8, bool);
///
///     fn from_sol_type(value: Self::SolType) -> Self {
///         Self {
///             size: value.0,
///             status: value.1,
///         }
///     }
/// }
/// ```
pub trait SolDecode {
    /// Equivalent Solidity ABI type representation.
    type SolType: SolTypeDecode;

    /// Name of equivalent Solidity ABI type.
    const SOL_NAME: &'static str =
        <<Self::SolType as SolTypeDecode>::AlloyType as AlloySolType>::SOL_NAME;

    /// Solidity ABI decode into this type.
    fn decode(data: &[u8]) -> Result<Self, alloy_sol_types::Error>
    where
        Self: Sized,
    {
        <Self::SolType as SolTypeDecode>::decode(data).map(Self::from_sol_type)
    }

    /// Converts to `Self` from `Self::SolType`.
    fn from_sol_type(value: Self::SolType) -> Self;
}

/// Maps an arbitrary Rust/ink! type to a Solidity ABI type equivalent for Solidity
/// ABI encoding.
///
/// # Note
/// Implementing this trait entails:
/// - Declaring the equivalent Solidity ABI type via the `SolType` associated type. See
///   the [docs for sealed `SolTypeEncode` trait][SolTypeEncode] for a table of Rust/ink!
///   primitive types mapped to their equivalent Solidity ABI type.
/// - Implementing the `to_sol_type` method which defines how to convert (preferably via a
///   borrow) from `&self` to `&Self::SolType` (i.e. the Solidity ABI representation).
///
/// # Example
///
/// ```
/// use ink_prelude::borrow::Cow;
/// use ink_primitives::SolEncode;
///
/// // Example arbitrary type.
/// struct MyType {
///     size: u8,
///     status: bool,
/// }
///
/// // `SolEncode` implementation/mapping.
/// impl SolEncode for MyType {
///     type SolType = (u8, bool);
///
///     fn to_sol_type(&self) -> Cow<(u8, bool)> {
///         // NOTE: Types that implement `Borrow<Self::SolType>`
///         // (e.g. newtype wrappers for `Self::SolType`)
///         // should return `Cow::Borrowed(self.borrow())` for better performance.
///         Cow::Owned((self.size, self.status))
///     }
/// }
/// ```
pub trait SolEncode {
    /// Equivalent Solidity ABI type representation.
    type SolType: SolTypeEncode + Clone;

    /// Name of equivalent Solidity ABI type.
    const SOL_NAME: &'static str =
        <<Self::SolType as SolTypeEncode>::AlloyType as AlloySolType>::SOL_NAME;

    /// Solidity ABI encode the value.
    fn encode(&self) -> Vec<u8> {
        <Self::SolType as SolTypeEncode>::encode(self.to_sol_type().deref())
    }

    /// Converts from `Self` to `Self::SolType` via either a borrow (if possible), or
    /// a possibly expensive conversion otherwise.
    fn to_sol_type(&self) -> Cow<Self::SolType>;
}

impl<T: SolDecode + Clone, const N: usize> SolDecode for [T; N] {
    type SolType = [T::SolType; N];

    fn from_sol_type(value: Self::SolType) -> Self {
        value.map(<T as SolDecode>::from_sol_type)
    }
}

impl<T: SolEncode + Clone, const N: usize> SolEncode for [T; N] {
    type SolType = [T::SolType; N];

    fn to_sol_type(&self) -> Cow<Self::SolType> {
        Cow::Owned(self.each_ref().map(|item| item.to_sol_type().into_owned()))
    }
}

impl<T: SolDecode> SolDecode for Vec<T> {
    type SolType = Vec<T::SolType>;

    fn from_sol_type(value: Self::SolType) -> Self {
        value
            .into_iter()
            .map(<T as SolDecode>::from_sol_type)
            .collect()
    }
}

impl<T: SolEncode> SolEncode for Vec<T> {
    type SolType = Vec<T::SolType>;

    fn to_sol_type(&self) -> Cow<Self::SolType> {
        Cow::Owned(
            self.iter()
                .map(|item| item.to_sol_type().into_owned())
                .collect(),
        )
    }
}

// We follow the Rust standard library's convention of implementing traits for tuples up
// to twelve items long.
// Ref: <https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations>
#[impl_for_tuples(12)]
impl SolDecode for Tuple {
    for_tuples!( type SolType = ( #( Tuple::SolType ),* ); );

    #[allow(clippy::unused_unit)]
    fn from_sol_type(value: Self::SolType) -> Self {
        for_tuples!( ( #( Tuple::from_sol_type(value.Tuple) ),* ) );
    }
}

#[impl_for_tuples(12)]
impl SolEncode for Tuple {
    for_tuples!( type SolType = ( #( Tuple::SolType ),* ); );

    fn to_sol_type(&self) -> Cow<Self::SolType> {
        Cow::Owned(for_tuples! ( ( #( self.Tuple.to_sol_type().into_owned() ),* ) ))
    }
}

// AccountId
impl SolDecode for AccountId {
    type SolType = AsSolBytes<[u8; 32]>;

    fn from_sol_type(value: Self::SolType) -> Self {
        AccountId(value.0)
    }
}

impl SolEncode for AccountId {
    type SolType = AsSolBytes<[u8; 32]>;

    fn encode(&self) -> Vec<u8> {
        // Override for better performance.
        sol_data::FixedBytes::abi_encode(self)
    }

    fn to_sol_type(&self) -> Cow<Self::SolType> {
        // NOTE: Not actually used for encoding because of `encode` override above (for
        // better performance).
        // Arbitrary newtype wrappers can achieve similar performance (without overriding
        // `encode`) by using  `AsSolBytes<[u8; 32]>` as the inner type and returning
        // `Cow::Borrowed(&self.0)`.
        Cow::Owned(AsSolBytes(self.0))
    }
}

// Hash
impl SolDecode for Hash {
    type SolType = AsSolBytes<[u8; 32]>;

    fn from_sol_type(value: Self::SolType) -> Self {
        Hash::from(value.0)
    }
}

impl SolEncode for Hash {
    type SolType = AsSolBytes<[u8; 32]>;

    fn encode(&self) -> Vec<u8> {
        // Override for better performance.
        sol_data::FixedBytes::abi_encode(self)
    }

    fn to_sol_type(&self) -> Cow<Self::SolType> {
        // NOTE: Not actually used for encoding because of `encode` override above (for
        // better performance).
        // Arbitrary newtype wrappers can achieve similar performance (without overriding
        // `encode`) by using  `AsSolBytes<[u8; 32]>` as the inner type and returning
        // `Cow::Borrowed(&self.0)`.
        Cow::Owned(AsSolBytes::<[u8; 32]>((*self).into()))
    }
}
