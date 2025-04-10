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
    abi::{
        self,
        token::{
            DynSeqToken,
            FixedSeqToken,
            PackedSeqToken,
            WordToken,
        },
    },
    private::SolTypeValue,
    sol_data,
    SolType as AlloySolType,
};
use core::ops::Deref;
use impl_trait_for_tuples::impl_for_tuples;
use ink_prelude::{
    borrow::Cow,
    boxed::Box,
    string::String,
    vec::Vec,
};
use itertools::Itertools;
use paste::paste;
use primitive_types::U256;

use crate::types::Address;

/// A Rust/ink! equivalent of a Solidity ABI type that implements logic for Solidity ABI
/// decoding.
///
/// # Rust/ink! to Solidity ABI type mapping
///
/// | Rust/ink! type | Solidity ABI type | Notes |
/// | -------------- | ----------------- | ----- |
/// | `bool` | `bool` ||
/// | `iN` for `N ∈ {8,16,32,64,128}` | `intN` | e.g `i8` ↔ `int8` |
/// | `uN` for `N ∈ {8,16,32,64,128}` | `uintN` | e.g `u8` ↔ `uint8` |
/// | `U256` | `uint256` ||
/// | `String` | `string` ||
/// | `Box<str>` | `string` ||
/// | `Address` / `H160` | `address` | `Address` is a type alias for the `H160` type used for addresses in `pallet-revive` |
/// | `[T; N]` for `const N: usize` | `T[N]` | e.g. `[i8; 64]` ↔ `int8[64]` |
/// | `Vec<T>` | `T[]` | e.g. `Vec<i8>` ↔ `int8[]` |
/// | `Box<[T]>` | `T[]` | e.g. `Box<[i8]>` ↔ `int8[]` |
/// | `SolBytes<u8>` | `bytes1` ||
/// | `SolBytes<[u8; N]>` for `1 <= N <= 32` | `bytesN` | e.g. `SolBytes<[u8; 32]>` ↔ `bytes32` |
/// | `SolBytes<Vec<u8>>` | `bytes` ||
/// | `SolBytes<Box<[u8]>>` | `bytes` ||
/// | `(T1, T2, T3, ... T12)` | `(U1, U2, U3, ... U12)` | where `T1` ↔ `U1`, ... `T12` ↔ `U12` e.g. `(bool, u8, Address)` ↔ `(bool, uint8, address)` |
///
/// Ref: <https://docs.soliditylang.org/en/latest/abi-spec.html#types>
///
/// # Note
///
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
#[allow(private_bounds)]
pub trait SolTypeDecode: Sized + private::Sealed {
    /// Equivalent Solidity ABI type from [`alloy_sol_types`].
    type AlloyType: AlloySolType;

    /// Solidity ABI decode into this type.
    fn decode(data: &[u8]) -> Result<Self, alloy_sol_types::Error> {
        abi::decode::<<Self::AlloyType as AlloySolType>::Token<'_>>(data)
            .and_then(Self::detokenize)
    }

    /// Detokenizes this type's value from the given token.
    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, alloy_sol_types::Error>;
}

/// A Rust/ink! equivalent of a Solidity ABI type that implements logic for Solidity ABI
/// encoding.
///
/// # Rust/ink! to Solidity ABI type mapping
///
/// | Rust/ink! type | Solidity ABI type | Notes |
/// | -------------- | ----------------- | ----- |
/// | `bool` | `bool` ||
/// | `iN` for `N ∈ {8,16,32,64,128}` | `intN` | e.g `i8` ↔ `int8` |
/// | `uN` for `N ∈ {8,16,32,64,128}` | `uintN` | e.g `u8` ↔ `uint8` |
/// | `U256` | `uint256` ||
/// | `String` | `string` ||
/// | `Box<str>` | `string` ||
/// | `Address` / `H160` | `address` | `Address` is a type alias for the `H160` type used for addresses in `pallet-revive` |
/// | `[T; N]` for `const N: usize` | `T[N]` | e.g. `[i8; 64]` ↔ `int8[64]` |
/// | `Vec<T>` | `T[]` | e.g. `Vec<i8>` ↔ `int8[]` |
/// | `Box<[T]>` | `T[]` | e.g. `Box<[i8]>` ↔ `int8[]` |
/// | `SolBytes<u8>` |  `bytes1` ||
/// | `SolBytes<[u8; N]>` for `1 <= N <= 32` | `bytesN` | e.g. `SolBytes<[u8; 32]>` ↔ `bytes32` |
/// | `SolBytes<Vec<u8>>` | `bytes` ||
/// | `SolBytes<Box<[u8]>>` | `bytes` ||
/// | `(T1, T2, T3, ... T12)` | `(U1, U2, U3, ... U12)` | where `T1` ↔ `U1`, ... `T12` ↔ `U12` e.g. `(bool, u8, Address)` ↔ `(bool, uint8, address)` |
/// | `&str`, `&mut str` | `string` ||
/// | `&T`, `&mut T`, `Box<T>` | `T` | e.g. `&i8 ↔ int8` |
/// | `&[T]`, `&mut [T]` | `T[]` | e.g. `&[i8]` ↔ `int8[]` |
///
/// Ref: <https://docs.soliditylang.org/en/latest/abi-spec.html#types>
///
/// # Note
///
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
#[allow(private_bounds)]
pub trait SolTypeEncode: private::Sealed {
    /// Equivalent Solidity ABI type from [`alloy_sol_types`].
    type AlloyType: AlloySolType;

    /// Solidity ABI encode the value.
    fn encode(&self) -> Vec<u8> {
        abi::encode(&self.tokenize())
    }

    /// Tokenizes the given value into a [`Self::AlloyType`] token.
    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_>;
}

macro_rules! impl_primitive_decode {
    ($($ty: ty => $sol_ty: ty),+ $(,)*) => {
        $(
            impl SolTypeDecode for $ty {
                type AlloyType = $sol_ty;

                fn detokenize(token: <Self::AlloyType as AlloySolType>::Token<'_>) -> Result<Self, alloy_sol_types::Error> {
                    Ok(<Self::AlloyType as AlloySolType>::detokenize(token))
                }
            }
        )*
    };
}

macro_rules! impl_primitive_encode {
    ($($ty: ty => $sol_ty: ty),+ $(,)*) => {
        $(
            impl SolTypeEncode for $ty where Self: SolTypeValue<$sol_ty> {
                type AlloyType = $sol_ty;

                fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
                    <Self::AlloyType as AlloySolType>::tokenize(self)
                }
            }
        )*
    };
}

macro_rules! impl_primitive {
    ($($ty: ty => $sol_ty: ty),+ $(,)*) => {
        $(
            impl_primitive_decode!($ty => $sol_ty);

            impl_primitive_encode!($ty => $sol_ty);

            impl private::Sealed for $ty {}
        )*
    };
}

macro_rules! impl_native_int {
    ($($bits: literal),+$(,)*) => {
        $(
            impl_primitive! {
                // signed
                paste!([<i $bits>]) => sol_data::Int<$bits>,
                // unsigned
                paste!([<u $bits>]) => sol_data::Uint<$bits>,
            }
        )*
    };
}

impl_native_int!(8, 16, 32, 64, 128);

impl_primitive! {
    // bool
    bool => sol_data::Bool,
    // string
    String => sol_data::String,
}

// Rust `Box<str>` (i.e. boxed string slice) <-> Solidity `string`.
impl SolTypeDecode for Box<str> {
    type AlloyType = sol_data::String;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, alloy_sol_types::Error> {
        Ok(Box::from(core::str::from_utf8(token.0).map_err(|_| {
            alloy_sol_types::Error::type_check_fail(token.0, "string")
        })?))
    }
}

impl SolTypeEncode for Box<str> {
    type AlloyType = sol_data::String;

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        PackedSeqToken(self.as_bytes())
    }
}

impl private::Sealed for Box<str> {}

// Address <-> address
impl SolTypeDecode for Address {
    type AlloyType = sol_data::Address;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, alloy_sol_types::Error> {
        // We skip the conversion to `alloy_sol_types::private::Address` which will end up
        // just taking the last 20 bytes of `alloy_sol_types::abi::token::WordToken` as
        // well anyway.
        // Ref: <https://github.com/alloy-rs/core/blob/5ae4fe0b246239602c97cc5a2f2e4bc780e2024a/crates/primitives/src/bits/address.rs#L132-L134>
        Ok(Address::from_slice(&token.0 .0[12..]))
    }
}

impl SolTypeEncode for Address {
    type AlloyType = sol_data::Address;

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        // We skip the conversion to `alloy_sol_types::private::Address` which will just
        // end up doing the conversion below anyway.
        // Ref: <https://github.com/alloy-rs/core/blob/5ae4fe0b246239602c97cc5a2f2e4bc780e2024a/crates/primitives/src/bits/address.rs#L149-L153>
        let mut word = [0; 32];
        word[12..].copy_from_slice(self.0.as_slice());
        WordToken::from(word)
    }
}

impl private::Sealed for Address {}

// U256 <-> uint256
impl SolTypeDecode for U256 {
    type AlloyType = sol_data::Uint<256>;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, alloy_sol_types::Error> {
        Ok(U256::from_big_endian(token.0 .0.as_slice()))
    }
}

impl SolTypeEncode for U256 {
    type AlloyType = sol_data::Uint<256>;

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        // `<Self::AlloyType as AlloySolType>::tokenize(self)` won't work because
        // `primitive_types::U256` does NOT implement
        // `Borrow<alloy_sol_types::private::U256>`. And both the `U256` and
        // `Borrow` are foreign, so we can't just implement it.
        WordToken::from(self.to_big_endian())
    }
}

impl private::Sealed for U256 {}

// Rust array <-> Solidity fixed-sized array (i.e. `T[N]`).
impl<T: SolTypeDecode, const N: usize> SolTypeDecode for [T; N] {
    type AlloyType = sol_data::FixedArray<T::AlloyType, N>;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, alloy_sol_types::Error> {
        // Takes advantage of optimized `SolTypeDecode::detokenize` implementations and
        // skips unnecessary conversions to `T::AlloyType::RustType`.
        // FIXME: (@davidsemakula) replace with `array::try_map` if it's ever stabilized.
        // Ref: <https://github.com/rust-lang/rust/issues/79711>
        // Ref: <https://doc.rust-lang.org/nightly/std/primitive.array.html#method.try_map>
        token
            .0
            .into_iter()
            .map(<T as SolTypeDecode>::detokenize)
            .process_results(|iter| iter.collect_array())?
            .ok_or_else(|| {
                alloy_sol_types::Error::custom(ink_prelude::format!(
                    "ABI decoding failed: {}",
                    Self::AlloyType::SOL_NAME
                ))
            })
    }
}

impl<T: SolTypeEncode, const N: usize> SolTypeEncode for [T; N] {
    type AlloyType = sol_data::FixedArray<T::AlloyType, N>;

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        // Does NOT require `SolValueType<Self::AlloyType>` and instead relies on
        // `SolTypeEncode::tokenize`.
        FixedSeqToken(core::array::from_fn(|i| {
            <T as SolTypeEncode>::tokenize(&self[i])
        }))
    }
}

impl<T: private::Sealed, const N: usize> private::Sealed for [T; N] {}

// Rust `Vec` <-> Solidity dynamic size array (i.e. `T[]`).
impl<T: SolTypeDecode> SolTypeDecode for Vec<T> {
    type AlloyType = sol_data::Array<T::AlloyType>;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, alloy_sol_types::Error> {
        // Takes advantage of optimized `SolTypeDecode::detokenize` implementations and
        // skips unnecessary conversions to `T::AlloyType::RustType`.
        token
            .0
            .into_iter()
            .map(<T as SolTypeDecode>::detokenize)
            .collect()
    }
}

impl<T: SolTypeEncode> SolTypeEncode for Vec<T> {
    type AlloyType = sol_data::Array<T::AlloyType>;

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        // Does NOT require `SolValueType<Self::AlloyType>` and instead relies on
        // `SolTypeEncode::tokenize`.
        DynSeqToken(self.iter().map(<T as SolTypeEncode>::tokenize).collect())
    }
}

impl<T: private::Sealed> private::Sealed for Vec<T> {}

// Rust `Box<[T]>` (i.e. boxed slice) <-> Solidity dynamic size array (i.e. `T[]`).
impl<T: SolTypeDecode> SolTypeDecode for Box<[T]> {
    type AlloyType = sol_data::Array<T::AlloyType>;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, alloy_sol_types::Error> {
        // Takes advantage of optimized `SolTypeDecode::detokenize` implementations and
        // skips unnecessary conversions to `T::AlloyType::RustType`.
        token
            .0
            .into_iter()
            .map(<T as SolTypeDecode>::detokenize)
            .collect()
    }
}

impl<T: SolTypeEncode> SolTypeEncode for Box<[T]> {
    type AlloyType = sol_data::Array<T::AlloyType>;

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        // Does NOT require `SolValueType<Self::AlloyType>` and instead relies on
        // `SolTypeEncode::tokenize`.
        DynSeqToken(self.iter().map(<T as SolTypeEncode>::tokenize).collect())
    }
}

impl<T: private::Sealed> private::Sealed for Box<[T]> {}

// We follow the Rust standard library's convention of implementing traits for tuples up
// to twelve items long.
// Ref: <https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations>
#[impl_for_tuples(12)]
impl SolTypeDecode for Tuple {
    for_tuples!( type AlloyType = ( #( Tuple::AlloyType ),* ); );

    #[allow(clippy::unused_unit)]
    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, alloy_sol_types::Error> {
        // Takes advantage of optimized `SolTypeDecode::detokenize` implementations and
        // skips unnecessary conversions to `T::AlloyType::RustType`.
        Ok(for_tuples! { ( #( <Tuple as SolTypeDecode>::detokenize(token.Tuple)? ),* ) })
    }
}

#[impl_for_tuples(12)]
impl SolTypeEncode for Tuple {
    for_tuples!( type AlloyType = ( #( Tuple::AlloyType ),* ); );

    #[allow(clippy::unused_unit)]
    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        // Does NOT require `SolValueType<Self::AlloyType>` and instead relies on
        // `SolTypeEncode::tokenize`.
        for_tuples!( ( #( <Tuple as SolTypeEncode>::tokenize(&self.Tuple) ),* ) );
    }
}

#[impl_for_tuples(12)]
impl private::Sealed for Tuple {}

// Implements `SolTypeEncode` for reference types.
macro_rules! impl_refs_encode {
    ($([$($gen:tt)*] $ty: ty), +$(,)*) => {
        $(

            impl<$($gen)* T: SolTypeEncode> SolTypeEncode for $ty {
                type AlloyType = T::AlloyType;

                fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
                    <T as SolTypeEncode>::tokenize(self)
                }
            }

            impl<$($gen)* T: private::Sealed> private::Sealed for $ty {}
        )*
    };
}

impl_refs_encode! {
    ['a,] &'a T,
    ['a,] &'a mut T,
    [] Box<T>,
}

impl<T: SolTypeEncode + Clone> SolTypeEncode for Cow<'_, T> {
    type AlloyType = T::AlloyType;

    fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
        <T as SolTypeEncode>::tokenize(self.deref())
    }
}

impl<T: private::Sealed + Clone> private::Sealed for Cow<'_, T> {}

// Implements `SolTypeEncode` for references to `str` and `[T]` DSTs.
macro_rules! impl_str_ref_encode {
    ($($ty: ty),+ $(,)*) => {
        $(
            impl SolTypeEncode for $ty {
                type AlloyType = sol_data::String;

                fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
                    PackedSeqToken(self.as_bytes())
                }
            }

            impl private::Sealed for $ty {}
        )*
    };
}

impl_str_ref_encode!(&str, &mut str);

macro_rules! impl_slice_ref_encode {
    ($($ty: ty),+ $(,)*) => {
        $(
            impl<T: SolTypeEncode> SolTypeEncode for $ty {
                type AlloyType = sol_data::Array<T::AlloyType>;

                fn tokenize(&self) -> <Self::AlloyType as AlloySolType>::Token<'_> {
                    // Does NOT require `SolValueType<Self::AlloyType>` and instead relies on
                    // `SolTypeEncode::tokenize`.
                    DynSeqToken(self.iter().map(<T as SolTypeEncode>::tokenize).collect())
                }
            }

            impl<T: private::Sealed> private::Sealed for $ty {}
        )*
    };
}

impl_slice_ref_encode!(&[T], &mut [T]);

pub(super) mod private {
    /// Seals implementations of `SolTypeEncode` and `SolTypeDecode`.
    pub trait Sealed {}
}
