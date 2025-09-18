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

#[macro_use]
mod macros;

mod bytes;
mod encodable;
mod error;
mod params;
mod result;
mod types;
mod utils;

#[cfg(test)]
mod tests;

use core::ops::Deref;

use alloy_sol_types::SolType as AlloySolType;
use impl_trait_for_tuples::impl_for_tuples;
use ink_prelude::{
    borrow::Cow,
    boxed::Box,
    string::String,
    vec::Vec,
};
use itertools::Itertools;
use primitive_types::{
    H256,
    U256,
};
use sp_weights::Weight;

pub use self::{
    bytes::{
        ByteSlice,
        DynBytes,
        FixedBytes,
    },
    error::{
        SolErrorDecode,
        SolErrorEncode,
    },
    params::{
        SolParamsDecode,
        SolParamsEncode,
    },
    result::{
        SolResultDecode,
        SolResultDecodeError,
        SolResultEncode,
    },
    types::{
        SolTopicEncode,
        SolTypeDecode,
        SolTypeEncode,
    },
};

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
/// use ink_primitives::{
///     SolDecode,
///     sol::Error,
/// };
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
///     fn from_sol_type(value: Self::SolType) -> Result<Self, Error> {
///         Ok(Self {
///             size: value.0,
///             status: value.1,
///         })
///     }
/// }
/// ```
pub trait SolDecode: Sized {
    /// Equivalent Solidity ABI type representation.
    type SolType: SolTypeDecode;

    /// Name of equivalent Solidity ABI type.
    const SOL_NAME: &'static str =
        <<Self::SolType as SolTypeDecode>::AlloyType as AlloySolType>::SOL_NAME;

    /// Solidity ABI decode into this type.
    fn decode(data: &[u8]) -> Result<Self, Error> {
        <Self::SolType as SolTypeDecode>::decode(data).and_then(Self::from_sol_type)
    }

    /// Converts to `Self` from `Self::SolType`.
    fn from_sol_type(value: Self::SolType) -> Result<Self, Error>;
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
    ///
    /// # Note
    ///
    /// Prefer reference based representation for better performance.
    type SolType: SolTypeEncode + SolTopicEncode;

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

    /// Solidity ABI encode the value as a topic (i.e. an indexed event parameter).
    fn encode_topic<H>(&'a self, hasher: H) -> [u8; 32]
    where
        H: Fn(&[u8], &mut [u8; 32]),
    {
        <Self::SolType as SolTopicEncode>::encode_topic(&self.to_sol_type(), hasher)
    }

    /// Converts from `Self` to `Self::SolType` via either a borrow (if possible), or
    /// a possibly expensive conversion otherwise.
    fn to_sol_type(&'a self) -> Self::SolType;
}

/// Solidity ABI encode the given value as a parameter sequence.
///
/// # Note
///
/// - `T` must be a tuple type where each member implements [`SolEncode`].
/// - The result can be different from [`SolEncode::encode`] for the given tuple because
///   this function always returns the encoded data in place, even for tuples containing
///   dynamic types (i.e. no offset is included for dynamic tuples).
///
/// This function is a convenience wrapper for [`SolParamsEncode::encode`].
pub fn encode_sequence<T: for<'a> SolParamsEncode<'a>>(value: &T) -> Vec<u8> {
    SolParamsEncode::encode(value)
}

/// Solidity ABI decode the given data as a parameter sequence.
///
/// # Note
///
/// - `T` must be a tuple type where each member implements [`SolDecode`].
/// - See notes for [`encode_sequence`] for the difference between this function and
///   [`SolDecode::decode`] for the given tuple.
pub fn decode_sequence<T: SolParamsDecode>(data: &[u8]) -> Result<T, Error> {
    SolParamsDecode::decode(data)
}

/// Solidity ABI encoding/decoding error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error;

impl From<alloy_sol_types::Error> for Error {
    fn from(_: alloy_sol_types::Error) -> Self {
        Self
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Solidity ABI encode/decode error")
    }
}

macro_rules! impl_primitive_decode {
    ($($ty: ty),+ $(,)*) => {
        $(
            impl SolDecode for $ty {
                type SolType = $ty;

                fn from_sol_type(value: Self::SolType) -> Result<Self, Error> {
                    Ok(value)
                }
            }
        )*
    };
}

macro_rules! impl_primitive_encode {
    ($($ty: ty),+ $(,)*) => {
        $(
            impl SolEncode<'_> for $ty {
                type SolType = $ty;

                fn to_sol_type(&self) -> Self::SolType {
                    *self
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

macro_rules! impl_primitive_encode_by_ref {
    ($($ty: ty, $ref_ty: ty),+ $(,)*) => {
        $(
            impl<'a> SolEncode<'a> for $ty {
                type SolType = &'a $ref_ty;

                fn to_sol_type(&'a self) -> Self::SolType {
                    self
                }
            }
        )*
    };
}

macro_rules! impl_primitive_by_ref {
    ($($ty: ty, $ref_ty: ty),+ $(,)*) => {
        $(
            impl_primitive_decode!($ty);

            impl_primitive_encode_by_ref!($ty, $ref_ty);
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
    // address
    Address,
}

impl_primitive_by_ref! {
    // string
    String, str,
    Box<str>, str,
}

// Rust array <-> Solidity fixed-sized array (i.e. `T[N]`).
impl<T: SolDecode, const N: usize> SolDecode for [T; N] {
    type SolType = [T::SolType; N];

    fn from_sol_type(value: Self::SolType) -> Result<Self, Error> {
        // FIXME: (@davidsemakula) replace with `array::try_map` if it's ever stabilized.
        // Ref: <https://github.com/rust-lang/rust/issues/79711>
        // Ref: <https://doc.rust-lang.org/nightly/std/primitive.array.html#method.try_map>
        value
            .into_iter()
            .map(<T as SolDecode>::from_sol_type)
            .process_results(|iter| iter.collect_array())?
            .ok_or(Error)
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

    fn from_sol_type(value: Self::SolType) -> Result<Self, Error> {
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

    fn from_sol_type(value: Self::SolType) -> Result<Self, Error> {
        value
            .into_iter()
            .map(<T as SolDecode>::from_sol_type)
            .process_results(|iter| iter.collect())
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
    fn from_sol_type(value: Self::SolType) -> Result<Self, Error> {
        Ok(for_tuples! { ( #( Tuple::from_sol_type(value.Tuple)? ),* ) })
    }
}

#[impl_for_tuples(12)]
impl<'a> SolEncode<'a> for Tuple {
    for_tuples!( type SolType = ( #( Tuple::SolType ),* ); );

    #[allow(clippy::unused_unit)]
    fn to_sol_type(&'a self) -> Self::SolType {
        for_tuples!( ( #( self.Tuple.to_sol_type() ),* ) )
    }
}

// Implements `SolEncode` for reference types.
macro_rules! impl_refs_encode {
    ($($ty: ty), +$(,)*) => {
        $(
            impl<'a, T> SolEncode<'a> for $ty
            where
                T: SolEncode<'a>,
            {
                type SolType = T::SolType;

                fn to_sol_type(&'a self) -> Self::SolType {
                    <T as SolEncode>::to_sol_type(self)
                }
            }
        )*
    };
}

impl_refs_encode! {
    &T,
    &mut T,
    Box<T>,
}

impl<'a, T> SolEncode<'a> for Cow<'_, T>
where
    T: SolEncode<'a> + Clone,
{
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

impl_str_ref_encode!(&str, &mut str);

macro_rules! impl_slice_ref_encode {
    ($($ty: ty),+ $(,)*) => {
        $(
            impl<'a, T> SolEncode<'a> for $ty
            where
                T: SolEncode<'a>,
            {
                type SolType = Vec<T::SolType>;

                fn to_sol_type(&'a self) -> Self::SolType {
                    self.iter().map(<T as SolEncode>::to_sol_type).collect()
                }
            }
        )*
    };
}

impl_slice_ref_encode!(&[T], &mut [T]);

// Option<T> <-> (bool, T)
//
// `bool` is a "flag" indicating the variant i.e. `false` for `None` and `true` for `Some`
// such that:
//  - `Option::None` is mapped to `(false, <default_value>)` where `<default_value>` is
//    the zero bytes only representation of `T` (e.g. `0u8` for `u8` or `Vec::<T>::new()`
//    for `Vec<T>`)
//  - `Option::Some(value)` is mapped to `(true, value)`
//
// # Note
//
// The resulting type in Solidity can be represented as struct with a field for the "flag"
// and another for the data.
//
// Note that `enum` in Solidity is encoded as `uint8` in Solidity ABI encoding, while the
// encoding for `bool` is equivalent to the encoding of `uint8` with `true` equivalent to
// `1` and `false` equivalent to `0`. Therefore, the `bool` "flag" can be safely
// interpreted as a `bool` or `enum` (or even `uint8`) in Solidity code.
//
// Ref: <https://docs.soliditylang.org/en/latest/abi-spec.html#mapping-solidity-to-abi-types>
impl<T> SolDecode for Option<T>
where
    T: SolDecode,
{
    type SolType = Option<T::SolType>;

    fn from_sol_type(value: Self::SolType) -> Result<Self, Error> {
        value.map(<T as SolDecode>::from_sol_type).transpose()
    }
}

impl<'a, T> SolEncode<'a> for Option<T>
where
    T: SolEncode<'a>,
{
    type SolType = Option<T::SolType>;

    fn to_sol_type(&'a self) -> Self::SolType {
        self.as_ref().map(T::to_sol_type)
    }
}

// Rust `PhantomData` <-> Solidity zero-tuple `()`.
impl<T> SolDecode for core::marker::PhantomData<T> {
    type SolType = ();

    fn decode(_: &[u8]) -> Result<Self, Error>
    where
        Self: Sized,
    {
        // NOTE: Solidity ABI decoding doesn't validate input length.
        Ok(core::marker::PhantomData)
    }

    fn from_sol_type(_: Self::SolType) -> Result<Self, Error> {
        Ok(core::marker::PhantomData)
    }
}

impl<T> SolEncode<'_> for core::marker::PhantomData<T> {
    type SolType = ();

    fn encode(&self) -> Vec<u8> {
        Vec::new()
    }

    fn to_sol_type(&self) {}
}

// AccountId <-> bytes32
impl SolDecode for AccountId {
    type SolType = FixedBytes<32>;

    fn from_sol_type(value: Self::SolType) -> Result<Self, Error> {
        Ok(AccountId(value.0))
    }
}

impl<'a> SolEncode<'a> for AccountId {
    type SolType = &'a FixedBytes<32>;

    fn to_sol_type(&'a self) -> Self::SolType {
        FixedBytes::from_ref(self.as_ref())
    }
}

// Hash <-> bytes32
impl SolDecode for Hash {
    type SolType = FixedBytes<32>;

    fn from_sol_type(value: Self::SolType) -> Result<Self, Error> {
        Ok(Hash::from(value.0))
    }
}

impl<'a> SolEncode<'a> for Hash {
    type SolType = &'a FixedBytes<32>;

    fn to_sol_type(&'a self) -> Self::SolType {
        use core::borrow::Borrow;
        FixedBytes::from_ref(self.borrow())
    }
}

// H256 <-> bytes32
impl SolDecode for H256 {
    type SolType = FixedBytes<32>;

    fn from_sol_type(value: Self::SolType) -> Result<Self, Error> {
        Ok(H256(value.0))
    }
}

impl<'a> SolEncode<'a> for H256 {
    type SolType = &'a FixedBytes<32>;

    fn to_sol_type(&'a self) -> Self::SolType {
        FixedBytes::from_ref(self.as_fixed_bytes())
    }
}

// Weight
impl SolDecode for Weight {
    type SolType = (u64, u64);

    fn from_sol_type(value: Self::SolType) -> Result<Self, Error> {
        Ok(Weight::from_parts(value.0, value.1))
    }
}

impl SolEncode<'_> for Weight {
    type SolType = (u64, u64);

    fn to_sol_type(&self) -> Self::SolType {
        (self.ref_time(), self.proof_size())
    }
}
