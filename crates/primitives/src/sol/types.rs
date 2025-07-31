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
    clone::Clone,
    ops::Deref,
};

use alloy_sol_types::{
    abi::{
        self,
        token::{
            PackedSeqToken,
            WordToken,
        },
        Encoder,
    },
    sol_data,
    SolType as AlloySolType,
};
use impl_trait_for_tuples::impl_for_tuples;
use ink_prelude::{
    borrow::{
        Cow,
        ToOwned,
    },
    boxed::Box,
    string::String,
    vec::Vec,
};
use itertools::Itertools;
use paste::paste;
use primitive_types::U256;

use crate::{
    sol::{
        encodable::{
            DynSizeDefault,
            Encodable,
            FixedSizeDefault,
            TokenOrDefault,
        },
        Error,
    },
    types::Address,
};

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
/// | `Option<T>` | `(bool, T)` | e.g. `Option<u8>` ↔ `(bool, uint8)`|
///
/// Ref: <https://docs.soliditylang.org/en/latest/abi-spec.html#types>
///
/// ## `Option<T>` representation
///
/// Rust's `Option<T>` type doesn't have a **semantically** equivalent Solidity ABI type,
/// because [Solidity enums][sol-enum] are field-less.
///
/// So `Option<T>` is mapped to a tuple representation instead (i.e. `(bool, T)`),
/// because this representation allows preservation of semantic information in Solidity,
/// by using the `bool` as a "flag" indicating the variant
/// (i.e. `false` for `None` and `true` for `Some`) such that:
/// - `Option::None` is mapped to `(false, <default_value>)`
///   where `<default_value>` is the zero bytes only representation of `T`
///   (e.g. `0u8` for `u8` or `Vec::new()` for `Vec<T>`)
/// - `Option::Some(value)` is mapped to `(true, value)`
///
/// The resulting type in Solidity can be represented as a struct with a field for the "flag"
/// and another for the data.
///
/// Note that `enum` in Solidity is encoded as `uint8` in [Solidity ABI encoding][sol-abi-types],
/// while the encoding for `bool` is equivalent to the encoding of `uint8`,
/// with `true` equivalent to `1` and `false` equivalent to `0`.
/// Therefore, the `bool` "flag" can be safely interpreted as a `bool` or `enum` (or even `uint8`)
/// in Solidity code.
///
/// [sol-enum]: https://docs.soliditylang.org/en/latest/types.html#enums
/// [sol-abi-types]: https://docs.soliditylang.org/en/latest/abi-spec.html#mapping-solidity-to-abi-types
///
/// # Note
///
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
pub trait SolTypeDecode: Sized + private::Sealed {
    /// Equivalent Solidity ABI type from [`alloy_sol_types`].
    type AlloyType: AlloySolType;

    /// Solidity ABI decode into this type.
    fn decode(data: &[u8]) -> Result<Self, Error> {
        abi::decode::<<Self::AlloyType as AlloySolType>::Token<'_>>(data)
            .map_err(Error::from)
            .and_then(Self::detokenize)
    }

    /// Detokenizes this type's value from the given token.
    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, Error>;
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
/// | `Option<T>` | `(bool, T)` | e.g. `Option<u8>` ↔ `(bool, uint8)`|
///
/// Ref: <https://docs.soliditylang.org/en/latest/abi-spec.html#types>
///
/// ## `Option<T>` representation
///
/// Rust's `Option<T>` type doesn't have a **semantically** equivalent Solidity ABI type,
/// because [Solidity enums][sol-enum] are field-less.
///
/// So `Option<T>` is mapped to a tuple representation instead (i.e. `(bool, T)`),
/// because this representation allows preservation of semantic information in Solidity,
/// by using the `bool` as a "flag" indicating the variant
/// (i.e. `false` for `None` and `true` for `Some`) such that:
/// - `Option::None` is mapped to `(false, <default_value>)`
///   where `<default_value>` is the zero bytes only representation of `T`
///   (e.g. `0u8` for `u8` or `Vec::new()` for `Vec<T>`)
/// - `Option::Some(value)` is mapped to `(true, value)`
///
/// The resulting type in Solidity can be represented as a struct with a field for the "flag"
/// and another for the data.
///
/// Note that `enum` in Solidity is encoded as `uint8` in [Solidity ABI encoding][sol-abi-types],
/// while the encoding for `bool` is equivalent to the encoding of `uint8`,
/// with `true` equivalent to `1` and `false` equivalent to `0`.
/// Therefore, the `bool` "flag" can be safely interpreted as a `bool` or `enum` (or even `uint8`)
/// in Solidity code.
///
/// [sol-enum]: https://docs.soliditylang.org/en/latest/types.html#enums
/// [sol-abi-types]: https://docs.soliditylang.org/en/latest/abi-spec.html#mapping-solidity-to-abi-types
///
/// # Note
///
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
pub trait SolTypeEncode: SolTokenType + private::Sealed {
    /// Equivalent Solidity ABI type from [`alloy_sol_types`].
    type AlloyType: AlloySolType;

    /// An encodable representation of the default value for this type.
    const DEFAULT_VALUE: Self::DefaultType;

    /// Solidity ABI encode the value.
    fn encode(&self) -> Vec<u8> {
        let token = self.tokenize();
        let mut encoder = Encoder::with_capacity(token.total_words());
        token.encode(&mut encoder);
        encoder.into_bytes()
    }

    /// Tokenizes the given value into a [`Self::AlloyType`] token.
    fn tokenize(&self) -> Self::TokenType<'_>;
}

/// Describes a "tokenizable" representation of [`SolTypeEncode`] implementing type.
///
/// # Note
///
/// The `TokenType` representation is similar to alloy types that implement `Token` and
/// `TokenSeq` traits, but is instead based on local trait [`Encodable`] which allows us
/// to implement it for custom abstractions that allow for "default" representations of
/// [`Self::TokenType`], most notably [`TokenOrDefault`].
//
// # Design Notes
//
// These abstractions are mainly necessary because the return type of
// [`alloy_sol_types::SolType::tokenize`] is encumbered by the lifetime of it's input.
//
// In the case of a "default" representation, this input would be an owned value defined
// in [`SolTypeEncode::tokenize`], and thus it's lifetime would be too short for the
// return type of [`SolTypeEncode::tokenize`] when using a `Token<'a>` based return type
// (i.e. `'a` would be lifetime of `&self`).
//
// Static references as solution are too cumbersome because:
// - `SolTypeEncode` implementing types are composable (i.e. arrays, `Vec`s and tuples of
//   `T: SolTypeEncode` implement `SolTypeEncode` generically)
// - Tokenizable default representations of some types are based on alloy types that use
//   "interior mutability" (e.g. the tokenizable default for `SolBytes<Vec<u8>>` would be
//   based on `alloy_primitives::bytes::Bytes`)
// Ref: <https://doc.rust-lang.org/reference/interior-mutability.html>
//
// Lastly, this trait only exists separate from `SolTypeEncode` so that the
// `TokenType<'enc>` GAT (Generic Associated Type) does NOT have a `where Self: 'a` bound
// which is too limiting for our use case.
//
// See <https://github.com/rust-lang/rust/issues/87479> for details.
pub trait SolTokenType: private::Sealed {
    /// The type of an encodable representation of this type.
    type TokenType<'enc>: Encodable;

    /// The type of an encodable "default" representation of this type.
    type DefaultType: Encodable;
}

macro_rules! impl_primitive_decode {
    ($($ty: ty => $sol_ty: ty),+ $(,)*) => {
        $(
            impl SolTypeDecode for $ty {
                type AlloyType = $sol_ty;

                fn detokenize(token: <Self::AlloyType as AlloySolType>::Token<'_>) -> Result<Self, Error> {
                    Ok(<Self::AlloyType as AlloySolType>::detokenize(token))
                }
            }
        )*
    };
}

macro_rules! impl_primitive_encode {
    ($($ty: ty => ($sol_ty: ty, $default_ty: ty, $default_value: expr)),+ $(,)*) => {
        $(
            impl SolTypeEncode for $ty {
                type AlloyType = $sol_ty;

                const DEFAULT_VALUE: Self::DefaultType = $default_value;

                fn tokenize(&self) -> Self::TokenType<'_> {
                    <Self::AlloyType as AlloySolType>::tokenize(self)
                }
            }

            impl SolTokenType for $ty {
                type TokenType<'enc> = <$sol_ty as AlloySolType>::Token<'enc>;

                type DefaultType = $default_ty;
            }
        )*
    };
}

macro_rules! impl_primitive {
    ($($ty: ty => ($sol_ty: ty, $default_ty: ty, $default_value: expr)),+ $(,)*) => {
        $(
            impl_primitive_decode!($ty => $sol_ty);

            impl_primitive_encode!($ty => ($sol_ty, $default_ty, $default_value));

            impl private::Sealed for $ty {}
        )*
    };
}

macro_rules! impl_native_int {
    ($($bits: literal),+$(,)*) => {
        $(
            impl_primitive! {
                // signed
                paste!([<i $bits>]) => (sol_data::Int<$bits>, FixedSizeDefault, FixedSizeDefault::WORD),
                // unsigned
                paste!([<u $bits>]) => (sol_data::Uint<$bits>, FixedSizeDefault, FixedSizeDefault::WORD),
            }
        )*
    };
}

impl_native_int!(8, 16, 32, 64, 128);

impl_primitive! {
    // bool
    bool => (sol_data::Bool, FixedSizeDefault, FixedSizeDefault::WORD),
    // string
    String => (sol_data::String, DynSizeDefault, DynSizeDefault),
}

// Rust `Box<str>` (i.e. boxed string slice) <-> Solidity `string`.
impl SolTypeDecode for Box<str> {
    type AlloyType = sol_data::String;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, Error> {
        Ok(Box::from(core::str::from_utf8(token.0).map_err(|_| Error)?))
    }
}

impl SolTypeEncode for Box<str> {
    type AlloyType = sol_data::String;

    const DEFAULT_VALUE: Self::DefaultType = DynSizeDefault;

    fn tokenize(&self) -> Self::TokenType<'_> {
        PackedSeqToken(self.as_bytes())
    }
}

impl SolTokenType for Box<str> {
    type TokenType<'enc> = PackedSeqToken<'enc>;

    type DefaultType = DynSizeDefault;
}

impl private::Sealed for Box<str> {}

// Address <-> address
impl SolTypeDecode for Address {
    type AlloyType = sol_data::Address;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, Error> {
        // We skip the conversion to `alloy_sol_types::private::Address` which will end up
        // just taking the last 20 bytes of `alloy_sol_types::abi::token::WordToken` as
        // well anyway.
        // Ref: <https://github.com/alloy-rs/core/blob/5ae4fe0b246239602c97cc5a2f2e4bc780e2024a/crates/primitives/src/bits/address.rs#L132-L134>
        Ok(Address::from_slice(&token.0 .0[12..]))
    }
}

impl SolTypeEncode for Address {
    type AlloyType = sol_data::Address;

    const DEFAULT_VALUE: Self::DefaultType = FixedSizeDefault::WORD;

    fn tokenize(&self) -> Self::TokenType<'_> {
        // We skip the conversion to `alloy_sol_types::private::Address` which will just
        // end up doing the conversion below anyway.
        // Ref: <https://github.com/alloy-rs/core/blob/5ae4fe0b246239602c97cc5a2f2e4bc780e2024a/crates/primitives/src/bits/address.rs#L149-L153>
        let mut word = [0; 32];
        word[12..].copy_from_slice(self.0.as_slice());
        WordToken::from(word)
    }
}

impl SolTokenType for Address {
    type TokenType<'enc> = WordToken;

    type DefaultType = FixedSizeDefault;
}

impl private::Sealed for Address {}

// U256 <-> uint256
impl SolTypeDecode for U256 {
    type AlloyType = sol_data::Uint<256>;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, Error> {
        Ok(U256::from_big_endian(token.0 .0.as_slice()))
    }
}

impl SolTypeEncode for U256 {
    type AlloyType = sol_data::Uint<256>;

    const DEFAULT_VALUE: Self::DefaultType = FixedSizeDefault::WORD;

    fn tokenize(&self) -> Self::TokenType<'_> {
        // `<Self::AlloyType as AlloySolType>::tokenize(self)` won't work because
        // `primitive_types::U256` does NOT implement
        // `Borrow<alloy_sol_types::private::U256>`. And both the `U256` and
        // `Borrow` are foreign, so we can't just implement it.
        WordToken::from(self.to_big_endian())
    }
}

impl SolTokenType for U256 {
    type TokenType<'enc> = WordToken;

    type DefaultType = FixedSizeDefault;
}

impl private::Sealed for U256 {}

// Rust array <-> Solidity fixed-sized array (i.e. `T[N]`).
impl<T: SolTypeDecode, const N: usize> SolTypeDecode for [T; N] {
    type AlloyType = sol_data::FixedArray<T::AlloyType, N>;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, Error> {
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
            .ok_or(Error)
    }
}

impl<T: SolTypeEncode, const N: usize> SolTypeEncode for [T; N] {
    type AlloyType = sol_data::FixedArray<T::AlloyType, N>;

    const DEFAULT_VALUE: Self::DefaultType = [T::DEFAULT_VALUE; N];

    fn tokenize(&self) -> Self::TokenType<'_> {
        // Does NOT require `SolTypeValue<Self::AlloyType>` and instead relies on
        // `SolTypeEncode::tokenize`.
        core::array::from_fn(|i| <T as SolTypeEncode>::tokenize(&self[i]))
    }
}

impl<T: SolTokenType, const N: usize> SolTokenType for [T; N] {
    type TokenType<'enc> = [T::TokenType<'enc>; N];

    type DefaultType = [T::DefaultType; N];
}

impl<T: private::Sealed, const N: usize> private::Sealed for [T; N] {}

// Rust `Vec` <-> Solidity dynamic size array (i.e. `T[]`).
impl<T: SolTypeDecode> SolTypeDecode for Vec<T> {
    type AlloyType = sol_data::Array<T::AlloyType>;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, Error> {
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

    const DEFAULT_VALUE: Self::DefaultType = DynSizeDefault;

    fn tokenize(&self) -> Self::TokenType<'_> {
        // Does NOT require `SolTypeValue<Self::AlloyType>` and instead relies on
        // `SolTypeEncode::tokenize`.
        self.iter().map(<T as SolTypeEncode>::tokenize).collect()
    }
}

impl<T: SolTokenType> SolTokenType for Vec<T> {
    type TokenType<'enc> = Vec<T::TokenType<'enc>>;

    type DefaultType = DynSizeDefault;
}

impl<T: private::Sealed> private::Sealed for Vec<T> {}

// Rust `Box<[T]>` (i.e. boxed slice) <-> Solidity dynamic size array (i.e. `T[]`).
impl<T: SolTypeDecode> SolTypeDecode for Box<[T]> {
    type AlloyType = sol_data::Array<T::AlloyType>;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, Error> {
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

    const DEFAULT_VALUE: Self::DefaultType = DynSizeDefault;

    fn tokenize(&self) -> Self::TokenType<'_> {
        // Does NOT require `SolTypeValue<Self::AlloyType>` and instead relies on
        // `SolTypeEncode::tokenize`.
        self.iter().map(<T as SolTypeEncode>::tokenize).collect()
    }
}

impl<T: SolTokenType> SolTokenType for Box<[T]> {
    type TokenType<'enc> = Vec<T::TokenType<'enc>>;

    type DefaultType = DynSizeDefault;
}

impl<T: private::Sealed> private::Sealed for Box<[T]> {}

// Rust `Cow<'_, [T]>` (i.e. clone-on-write slice) <-> Solidity dynamic size array (i.e.
// `T[]`).
impl<T> SolTypeDecode for Cow<'_, [T]>
where
    T: SolTypeDecode + Clone,
    [T]: ToOwned,
{
    type AlloyType = sol_data::Array<T::AlloyType>;

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, Error> {
        // Takes advantage of optimized `SolTypeDecode::detokenize` implementations and
        // skips unnecessary conversions to `T::AlloyType::RustType`.
        token
            .0
            .into_iter()
            .map(<T as SolTypeDecode>::detokenize)
            .collect()
    }
}

impl<T> SolTypeEncode for Cow<'_, [T]>
where
    T: SolTypeEncode + Clone,
    [T]: ToOwned,
{
    type AlloyType = sol_data::Array<T::AlloyType>;

    const DEFAULT_VALUE: Self::DefaultType = DynSizeDefault;

    fn tokenize(&self) -> Self::TokenType<'_> {
        // Does NOT require `SolTypeValue<Self::AlloyType>` and instead relies on
        // `SolTypeEncode::tokenize`.
        self.iter().map(<T as SolTypeEncode>::tokenize).collect()
    }
}

impl<T> SolTokenType for Cow<'_, [T]>
where
    T: SolTokenType + Clone,
    [T]: ToOwned,
{
    type TokenType<'enc> = Vec<T::TokenType<'enc>>;

    type DefaultType = DynSizeDefault;
}

impl<T: private::Sealed> private::Sealed for Cow<'_, [T]> where [T]: ToOwned {}

// We follow the Rust standard library's convention of implementing traits for tuples up
// to twelve items long.
// Ref: <https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations>
#[impl_for_tuples(12)]
impl SolTypeDecode for Tuple {
    for_tuples!( type AlloyType = ( #( Tuple::AlloyType ),* ); );

    #[allow(clippy::unused_unit)]
    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, Error> {
        // Takes advantage of optimized `SolTypeDecode::detokenize` implementations and
        // skips unnecessary conversions to `T::AlloyType::RustType`.
        Ok(for_tuples! { ( #( <Tuple as SolTypeDecode>::detokenize(token.Tuple)? ),* ) })
    }
}

#[impl_for_tuples(12)]
impl SolTypeEncode for Tuple {
    for_tuples!( type AlloyType = ( #( Tuple::AlloyType ),* ); );

    const DEFAULT_VALUE: Self::DefaultType =
        (for_tuples! { #( Tuple::DEFAULT_VALUE ),* });

    #[allow(clippy::unused_unit)]
    fn tokenize(&self) -> Self::TokenType<'_> {
        // Does NOT require `SolTypeValue<Self::AlloyType>` and instead relies on
        // `SolTypeEncode::tokenize`.
        for_tuples!( ( #( <Tuple as SolTypeEncode>::tokenize(&self.Tuple) ),* ) );
    }
}

// `impl-trait-for-tuples` doesn't support GATs yet, so we fallback to a declarative macro
// for `SolTokenType`.
//
// Ref: <https://github.com/bkchr/impl-trait-for-tuples/issues/11>
macro_rules! impl_sol_token_type {
    ( $($ty: ident),* ) => {
        impl<$( $ty: SolTokenType, )*> SolTokenType for ( $( $ty, )* ) {
            type TokenType<'enc> = ( $( $ty ::TokenType<'enc>, )* );

            type DefaultType = ( $( $ty ::DefaultType, )* );
        }
    };
}

impl_all_tuples!(impl_sol_token_type);

#[impl_for_tuples(12)]
impl private::Sealed for Tuple {}

// Rust `Option<T>` <-> Solidity `(bool, T)`.
impl<T: SolTypeDecode> SolTypeDecode for Option<T> {
    type AlloyType = (sol_data::Bool, T::AlloyType);

    fn detokenize(
        token: <Self::AlloyType as AlloySolType>::Token<'_>,
    ) -> Result<Self, Error> {
        let flag = bool::detokenize(token.0)?;
        let value = T::detokenize(token.1)?;
        Ok(if flag { Some(value) } else { None })
    }
}

impl<T> SolTypeEncode for Option<T>
where
    T: SolTypeEncode,
{
    type AlloyType = (sol_data::Bool, T::AlloyType);

    const DEFAULT_VALUE: Self::DefaultType = (FixedSizeDefault::WORD, T::DEFAULT_VALUE);

    fn tokenize(&self) -> Self::TokenType<'_> {
        match self {
            Some(value) => (true.tokenize(), TokenOrDefault::Token(value.tokenize())),
            None => (false.tokenize(), TokenOrDefault::Default(T::DEFAULT_VALUE)),
        }
    }
}

impl<T: SolTokenType> SolTokenType for Option<T> {
    type TokenType<'enc> = (
        WordToken,
        TokenOrDefault<T::TokenType<'enc>, T::DefaultType>,
    );

    type DefaultType = (FixedSizeDefault, T::DefaultType);
}

impl<T: private::Sealed> private::Sealed for Option<T> {}

// Implements `SolTypeEncode` for reference types.
macro_rules! impl_refs_encode {
    ($($ty: ty), +$(,)*) => {
        $(
            impl<T: SolTypeEncode> SolTypeEncode for $ty {
                type AlloyType = T::AlloyType;

                const DEFAULT_VALUE: Self::DefaultType = T::DEFAULT_VALUE;

                fn tokenize(&self) -> Self::TokenType<'_> {
                    <T as SolTypeEncode>::tokenize(self)
                }
            }

            impl<T: SolTokenType> SolTokenType for $ty {
                type TokenType<'enc> = T::TokenType<'enc>;

                type DefaultType = T::DefaultType;
            }

            impl<T: private::Sealed> private::Sealed for $ty {}
        )*
    };
}

impl_refs_encode! {
    &T,
    &mut T,
    Box<T>,
}

// Implements `SolTypeEncode` for smart pointers.
impl<T: SolTypeEncode + Clone> SolTypeEncode for Cow<'_, T> {
    type AlloyType = T::AlloyType;

    const DEFAULT_VALUE: Self::DefaultType = T::DEFAULT_VALUE;

    fn tokenize(&self) -> Self::TokenType<'_> {
        <T as SolTypeEncode>::tokenize(self.deref())
    }
}

impl<T: SolTokenType + Clone> SolTokenType for Cow<'_, T> {
    type TokenType<'enc> = T::TokenType<'enc>;

    type DefaultType = T::DefaultType;
}

impl<T: private::Sealed + Clone> private::Sealed for Cow<'_, T> {}

// Implements `SolTypeEncode` for references and smart pointers to `str`.
macro_rules! impl_str_ref_encode {
    ($($ty: ty),+ $(,)*) => {
        $(
            impl SolTypeEncode for $ty {
                type AlloyType = sol_data::String;

                const DEFAULT_VALUE: Self::DefaultType = DynSizeDefault;

                fn tokenize(&self) -> Self::TokenType<'_> {
                    PackedSeqToken(self.as_bytes())
                }
            }

            impl SolTokenType for $ty {
                type TokenType<'enc> = PackedSeqToken<'enc>;

                type DefaultType = DynSizeDefault;
            }

            impl private::Sealed for $ty {}
        )*
    };
}

impl_str_ref_encode!(&str, &mut str, Cow<'_, str>);

// Implements `SolTypeEncode` for references to `[T]` DSTs.
macro_rules! impl_slice_ref_encode {
    ($($ty: ty),+ $(,)*) => {
        $(
            impl<T: SolTypeEncode> SolTypeEncode for $ty {
                type AlloyType = sol_data::Array<T::AlloyType>;

                const DEFAULT_VALUE: Self::DefaultType = DynSizeDefault;

                fn tokenize(&self) -> Self::TokenType<'_> {
                    // Does NOT require `SolTypeValue<Self::AlloyType>` and instead relies on
                    // `SolTypeEncode::tokenize`.
                    self.iter().map(<T as SolTypeEncode>::tokenize).collect()
                }
            }

            impl<T: SolTokenType> SolTokenType for $ty {
                type TokenType<'enc> = Vec<T::TokenType<'enc>>;

                type DefaultType = DynSizeDefault;
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
