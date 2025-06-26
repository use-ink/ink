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
mod params;
mod types;

#[cfg(test)]
mod tests;

use core::ops::Deref;

use alloy_sol_types::{
    sol_data,
    SolType as AlloySolType,
};
use impl_trait_for_tuples::impl_for_tuples;
use ink_prelude::{
    borrow::Cow,
    boxed::Box,
    string::String,
    vec::Vec,
};
use primitive_types::{
    H256,
    U256,
};

pub use self::{
    bytes::SolBytes,
    params::{
        SolParamsDecode,
        SolParamsEncode,
    },
    types::{
        SolTypeDecode,
        SolTypeEncode,
    },
};
pub use alloy_sol_types::Error;

use crate::types::{
    AccountId,
    Address,
    Hash,
};

/// Maps an arbitrary Rust/ink! type to a Solidity ABI type equivalent for Solidity
/// ABI decoding.
///
/// # Note
///
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
///
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
/// use ink_primitives::SolEncode;
///
/// // Example arbitrary type.
/// struct MyType {
///     size: u8,
///     status: bool,
/// }
///
/// // `SolEncode` implementation/mapping.
/// impl<'a> SolEncode<'a> for MyType {
///     // NOTE: Prefer reference based representation for better performance.
///     type SolType = (&'a u8, &'a bool);
///
///     fn to_sol_type(&'a self) -> Self::SolType {
///         (&self.size, &self.status)
///     }
/// }
/// ```
pub trait SolEncode<'a> {
    /// Equivalent Solidity ABI type representation.
    type SolType: SolTypeEncode;

    /// Name of equivalent Solidity ABI type.
    const SOL_NAME: &'static str =
        <<Self::SolType as SolTypeEncode>::AlloyType as AlloySolType>::SOL_NAME;

    /// Whether the ABI encoded size is dynamic.
    #[doc(hidden)]
    const DYNAMIC: bool =
        <<Self::SolType as SolTypeEncode>::AlloyType as AlloySolType>::DYNAMIC;

    /// Solidity ABI encode the value.
    fn encode(&'a self) -> Vec<u8> {
        <Self::SolType as SolTypeEncode>::encode(&self.to_sol_type())
    }

    /// Converts from `Self` to `Self::SolType` via either a borrow (if possible), or
    /// a possibly expensive conversion otherwise.
    fn to_sol_type(&'a self) -> Self::SolType;
}

macro_rules! impl_primitive_decode {
    ($($ty: ty),+ $(,)*) => {
        $(
            impl SolDecode for $ty {
                type SolType = $ty;

                fn from_sol_type(value: Self::SolType) -> Self {
                    value
                }
            }
        )*
    };
}

macro_rules! impl_primitive_encode {
    ($($ty: ty),+ $(,)*) => {
        $(
            impl<'a> SolEncode<'a> for $ty {
                type SolType = &'a $ty;

                fn to_sol_type(&'a self) -> Self::SolType {
                    self
                }
            }
        )*
    };
}

macro_rules! impl_primitive {
    ($($ty: ty),+ $(,)*) => {
        $(
            impl_primitive_decode!($ty);

            impl_primitive_encode!($ty);
        )*
    };
}

impl_primitive! {
    // bool
    bool,
    // signed integers
    i8, i16, i32, i64, i128,
    // unsigned integers
    u8, u16, u32, u64, u128, U256,
    // string
    String,
    Box<str>,
    // address
    Address,
}

// Rust array <-> Solidity fixed-sized array (i.e. `T[N]`).
impl<T: SolDecode, const N: usize> SolDecode for [T; N] {
    type SolType = [T::SolType; N];

    fn from_sol_type(value: Self::SolType) -> Self {
        value.map(<T as SolDecode>::from_sol_type)
    }
}

impl<'a, T: SolEncode<'a>, const N: usize> SolEncode<'a> for [T; N] {
    type SolType = [T::SolType; N];

    fn to_sol_type(&'a self) -> Self::SolType {
        self.each_ref().map(<T as SolEncode>::to_sol_type)
    }
}

// Rust `Vec` <-> Solidity dynamic size array (i.e. `T[]`).
impl<T: SolDecode> SolDecode for Vec<T> {
    type SolType = Vec<T::SolType>;

    fn from_sol_type(value: Self::SolType) -> Self {
        value
            .into_iter()
            .map(<T as SolDecode>::from_sol_type)
            .collect()
    }
}

impl<'a, T: SolEncode<'a>> SolEncode<'a> for Vec<T> {
    type SolType = Vec<T::SolType>;

    fn to_sol_type(&'a self) -> Self::SolType {
        self.iter().map(<T as SolEncode>::to_sol_type).collect()
    }
}

// Rust `Box<[T]>` (i.e. boxed slice) <-> Solidity dynamic size array (i.e. `T[]`).
impl<T: SolDecode> SolDecode for Box<[T]> {
    type SolType = Box<[T::SolType]>;

    fn from_sol_type(value: Self::SolType) -> Self {
        // TODO: (@davidsemakula) Switch to method call syntax when edition is 2024
        // (i.e. `value.into_iter()`).
        // See <https://doc.rust-lang.org/edition-guide/rust-2024/intoiterator-box-slice.html> for details.
        Box::from_iter(
            core::iter::IntoIterator::into_iter(value)
                .map(<T as SolDecode>::from_sol_type),
        )
    }
}

impl<'a, T: SolEncode<'a>> SolEncode<'a> for Box<[T]> {
    type SolType = Box<[T::SolType]>;

    fn to_sol_type(&'a self) -> Self::SolType {
        self.iter().map(<T as SolEncode>::to_sol_type).collect()
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
impl<'a> SolEncode<'a> for Tuple {
    for_tuples!( type SolType = ( #( Tuple::SolType ),* ); );

    #[allow(clippy::unused_unit)]
    fn to_sol_type(&'a self) -> Self::SolType {
        for_tuples! ( ( #( self.Tuple.to_sol_type() ),* ) )
    }
}

// Implements `SolEncode` for reference types.
macro_rules! impl_refs_encode {
    ($([$($gen:tt)*] $ty: ty), +$(,)*) => {
        $(
            impl<$($gen)* T: SolEncode<'a>> SolEncode<'a> for $ty {
                type SolType = T::SolType;

                fn to_sol_type(&'a self) -> Self::SolType {
                    <T as SolEncode>::to_sol_type(self)
                }
            }
        )*
    };
}

impl_refs_encode! {
    ['a,] &'a T,
    ['a,] &'a mut T,
    ['a,] Box<T>,
}

impl<'a, T: SolEncode<'a> + Clone> SolEncode<'a> for Cow<'a, T> {
    type SolType = T::SolType;

    fn to_sol_type(&'a self) -> Self::SolType {
        <T as SolEncode>::to_sol_type(self.deref())
    }
}

// Implements `SolEncode` for references to `str` and `[T]` DSTs.
macro_rules! impl_str_ref_encode {
    ($($ty: ty),+ $(,)*) => {
        $(
            impl<'a> SolEncode<'a> for $ty {
                type SolType = &'a str;

                fn to_sol_type(&'a self) -> Self::SolType {
                    self
                }
            }
        )*
    };
}

impl_str_ref_encode!(&'a str, &'a mut str);

macro_rules! impl_slice_ref_encode {
    ($($ty: ty),+ $(,)*) => {
        $(
            impl<'a, T: SolEncode<'a>> SolEncode<'a> for $ty {
                type SolType = Vec<T::SolType>;

                fn to_sol_type(&'a self) -> Self::SolType {
                    self.iter().map(<T as SolEncode>::to_sol_type).collect()
                }
            }
        )*
    };
}

impl_slice_ref_encode!(&'a [T], &'a mut [T]);

// AccountId <-> bytes32
impl SolDecode for AccountId {
    type SolType = SolBytes<[u8; 32]>;

    fn from_sol_type(value: Self::SolType) -> Self {
        AccountId(value.0)
    }
}

impl SolEncode<'_> for AccountId {
    type SolType = SolBytes<[u8; 32]>;

    fn encode(&self) -> Vec<u8> {
        // Override for better performance.
        sol_data::FixedBytes::abi_encode(self)
    }

    fn to_sol_type(&self) -> Self::SolType {
        // NOTE: Not actually used for encoding because of `encode` override above (for
        // better performance).
        // Arbitrary newtype wrappers can achieve similar performance (without overriding
        // `encode`) by using `SolBytes<[u8; 32]>` as the inner type and returning
        // `&self.0`.
        SolBytes(self.0)
    }
}

// Hash <-> bytes32
impl SolDecode for Hash {
    type SolType = SolBytes<[u8; 32]>;

    fn from_sol_type(value: Self::SolType) -> Self {
        Hash::from(value.0)
    }
}

impl SolEncode<'_> for Hash {
    type SolType = SolBytes<[u8; 32]>;

    fn encode(&self) -> Vec<u8> {
        // Override for better performance.
        sol_data::FixedBytes::abi_encode(self)
    }

    fn to_sol_type(&self) -> Self::SolType {
        // NOTE: Not actually used for encoding because of `encode` override above (for
        // better performance).
        // Arbitrary newtype wrappers can achieve similar performance (without overriding
        // `encode`) by using `SolBytes<[u8; 32]>` as the inner type and returning
        // `&self.0`.
        SolBytes::<[u8; 32]>((*self).into())
    }
}

// H256 <-> bytes32
impl SolDecode for H256 {
    type SolType = SolBytes<[u8; 32]>;

    fn from_sol_type(value: Self::SolType) -> Self {
        H256(value.0)
    }
}

impl SolEncode<'_> for H256 {
    type SolType = SolBytes<[u8; 32]>;

    fn encode(&self) -> Vec<u8> {
        // Override for better performance.
        sol_data::FixedBytes::abi_encode(&self.0)
    }

    fn to_sol_type(&self) -> Self::SolType {
        // NOTE: Not actually used for encoding because of `encode` override above (for
        // better performance).
        // Arbitrary newtype wrappers can achieve similar performance (without overriding
        // `encode`) by using `SolBytes<[u8; 32]>` as the inner type and returning
        // `&self.0`.
        SolBytes(self.0)
    }
}
