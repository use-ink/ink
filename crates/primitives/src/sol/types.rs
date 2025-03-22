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
use impl_trait_for_tuples::impl_for_tuples;
use ink_prelude::{
    borrow::Cow,
    string::String,
    vec::Vec,
};
use paste::paste;

use crate::{
    sol::{
        from::SolFrom,
        SolCodec,
    },
    types::Address,
};

/// A Rust/ink! equivalent of a Solidity ABI type that implements logic for Solidity ABI
/// encoding/decoding.
///
/// | Rust/ink! type | Solidity ABI type | Notes |
/// | -------------- | ----------------- | ----- |
/// | `bool` | `bool` ||
/// | `iN` for `N ∈ {8,16,32,64,128}` | `intN` | e.g `i8` <=> `int8` |
/// | `uN` for `N ∈ {8,16,32,64,128}` | `uintN` | e.g `u8` <=> `uint8` |
/// | `String` | `string` ||
/// | `Address` | `address` ||
/// | `[T; N]` for `const N: usize` | `T[N]` | e.g. `[i8; 64]` <=> `int8[64]` |
/// | `Vec<T>` | `T[]` | e.g. `Vec<i8>` <=> `int8[]` |
/// | `AsSolBytes<[u8; N]>` for `1 <= N <= 32` |  `bytesN` | e.g. `AsSolBytes<[u8; 1]>` <=> `bytes1` |
/// | `AsSolBytes<Vec<u8>>` |  `bytes` ||
/// | `(T1, T2, T3, ... T12)` | `(U1, U2, U3, ... U12)` | where `T1` <=> `U1`, ... `T12` <=> `U12` e.g. `(bool, u8, Address)` <=> `(bool, uint8, address)` |
///
/// Ref: <https://docs.soliditylang.org/en/latest/abi-spec.html#types>
///
/// # Note
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
#[allow(private_bounds)]
pub trait SolType:
    SolTypeValue<Self::AlloyType>
    + SolFrom<<<Self as SolType>::AlloyType as AlloySolType>::RustType>
    + private::Sealed
{
    /// Equivalent Solidity ABI type from [`alloy_sol_types`].
    type AlloyType: AlloySolType;

    /// Solidity ABI encode the value.
    fn encode(&self) -> Vec<u8> {
        <Self::AlloyType as AlloySolType>::abi_encode(self)
    }

    /// Solidity ABI decode into this type.
    fn decode(data: &[u8]) -> Result<Self, alloy_sol_types::Error> {
        // Don't validate decoding. Validating results in encoding and decoding again.
        <Self::AlloyType as AlloySolType>::abi_decode(data, false).map(Self::sol_from)
    }
}

macro_rules! impl_primitive {
    ($($ty: ty => $sol_ty: ty),+ $(,)*) => {
        $(
            impl SolType for $ty {
                type AlloyType = $sol_ty;
            }

            impl SolCodec for $ty {
                type SolType = $ty;

                fn to_sol_type(&self) -> Cow<Self::SolType> {
                    Cow::Borrowed(self)
                }

                fn from_sol_type(value: Self::SolType) -> Self {
                    value
                }
            }

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
    //str => sol_data::String,
    String => sol_data::String,
    // address
    Address => sol_data::Address,
}

macro_rules! impl_generics {
    ($([$($gen:tt)+] $ty: ty => $sol_ty: ty [$($bounds:tt)*]), +$(,)*) => {
        $(
        impl<$($gen)*> SolType for $ty where
        Self: SolFrom<<$sol_ty as AlloySolType>::RustType>, $($bounds)*
        {
            type AlloyType = $sol_ty;
        }

        impl<$($gen)*> private::Sealed for $ty {}
        )*
    };
}

impl_generics! {
    // array
    [T: SolType, const N: usize] [T; N] => sol_data::FixedArray<T::AlloyType, N> [],
    //[T: SolType] [T] => sol_data::Array<T::AlloyType> [],
    [T: SolType] Vec<T> => sol_data::Array<T::AlloyType> [],
    // references
    ['a, T: SolType] &'a T => T::AlloyType [&'a T: SolTypeValue<T::AlloyType>],
    ['a, T: SolType] &'a mut T => T::AlloyType [&'a mut T: SolTypeValue<T::AlloyType>],
}

// We follow the Rust standard library's convention of implementing traits for tuples up
// to twelve items long.
// Ref: <https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations>
#[impl_for_tuples(12)]
impl SolType for Tuple {
    for_tuples!( type AlloyType = ( #( Tuple::AlloyType ),* ); );
}

#[impl_for_tuples(12)]
impl private::Sealed for Tuple {}

pub(super) mod private {
    /// Seals implementations of `SolType`.
    pub trait Sealed {}
}
