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

use alloy_sol_types::{
    private::SolTypeValue,
    sol_data,
    SolType,
};
use impl_trait_for_tuples::impl_for_tuples;
use paste::paste;

/// A Rust equivalent of a Solidity type that implements logic for Solidity ABI
/// encoding/decoding.
///
/// Ref: <https://docs.soliditylang.org/en/latest/abi-spec.html#types>
///
/// # Note
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
pub trait SolPrimitive: SolTypeValue<Self::SolType> + private::Sealed {
    /// Equivalent Solidity type from [`alloy_sol_types`].
    type SolType: SolType;

    /// Name of equivalent Solidity type.
    const SOL_NAME: &'static str = <Self::SolType as SolType>::SOL_NAME;

    /// Solidity ABI encode the value.
    fn encode(&self) -> Vec<u8> {
        Self::SolType::abi_encode(self)
    }

    /// Solidity ABI decode into this type.
    fn decode(data: &[u8]) -> Result<Self, alloy_sol_types::Error>
    where
        Self: From<<Self::SolType as SolType>::RustType>,
    {
        Self::SolType::abi_decode(data, false).map(Self::from)
    }
}

/// Marker trait implemented by all Solidity type equivalents except `u8`.
///
/// # Note
/// See <https://github.com/use-ink/ink/issues/2428> for motivation.
trait NonU8: SolPrimitive {}

macro_rules! impl_primitive_minimal {
    ($($ty: ty => $sol_ty: ty),+ $(,)*) => {
        $(
            impl SolPrimitive for $ty {
                type SolType = $sol_ty;
            }

            impl private::Sealed for $ty {}
        )*
    };
}

macro_rules! impl_primitive {
    ($($ty: ty => $sol_ty: ty),+ $(,)*) => {
        $(
            impl_primitive_minimal!($ty => $sol_ty);

            impl NonU8 for $ty {}
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

impl_primitive!(i8 => sol_data::Int<8>);
impl_native_int!(16, 32, 64, 128);

// `u8` requires special handling, so we special case by implementing `NonU8` for all
// other types.
// See <https://github.com/use-ink/ink/issues/2428> for motivation.
impl_primitive_minimal!(u8 => sol_data::Uint<8>);

impl_primitive! {
    // bool
    bool => sol_data::Bool,
    // string
    str => sol_data::String,
    String => sol_data::String,
    // bytes
    // `u8` sequences/collections are mapped to `bytes` equivalents.
    [u8] => sol_data::Bytes,
    Vec<u8> => sol_data::Bytes,
}

macro_rules! impl_generics {
    ($([$($gen:tt)+] $ty: ty => $sol_ty: ty [$($where:tt)*]), +$(,)*) => {
        $(
        impl<$($gen)*> SolPrimitive for $ty $($where)* {
            type SolType = $sol_ty;
        }

        impl<$($gen)*> NonU8 for $ty $($where)* {}

        impl<$($gen)*> private::Sealed for $ty $($where)* {}
        )*
    };
}

impl_generics! {
    // byte array
    [const N: usize] [u8; N] => sol_data::FixedBytes<N> [where sol_data::ByteCount<N>: sol_data::SupportedFixedBytes],
    // array
    [T: NonU8, const N: usize] [T; N] => sol_data::FixedArray<T::SolType, N> [],
    [T: NonU8] [T] => sol_data::Array<T::SolType> [],
    [T: NonU8] Vec<T> => sol_data::Array<T::SolType> [],
    // references
    ['a, T: ?Sized + SolPrimitive] &'a T => T::SolType [where &'a T: SolTypeValue<T::SolType>],
    ['a, T: ?Sized + SolPrimitive] &'a mut T => T::SolType [where &'a mut T: SolTypeValue<T::SolType>],
}

// We follow the Rust standard library's convention of implementing traits for tuples up
// to twelve items long.
// Ref: <https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations>
#[impl_for_tuples(12)]
impl SolPrimitive for Tuple {
    for_tuples!( type SolType = ( #( Tuple::SolType ),* ); );
}

#[impl_for_tuples(12)]
impl private::Sealed for Tuple {}

#[impl_for_tuples(12)]
impl NonU8 for Tuple {}

mod private {
    /// Seals the implementations of `SolPrimitive` and `NonU8`.
    pub trait Sealed {}
}

/// Sanity checks to ensure `SolPrimitive` implementations match `SolValue` equivalents.
#[cfg(test)]
mod tests {
    use super::*;
    use alloy_sol_types::{
        sol_data::Uint,
        SolValue,
    };

    macro_rules! test_case {
        ($ty: ty, $val: expr) => {
            test_case!($ty, $val, $ty, alloy_sol_types::SolValue, $val, [], [])
        };
        ($ty: ty, $val: expr, $sol_ty: ty, $sol_trait: ty) => {
            test_case!($ty, $val, $sol_ty, $sol_trait, $val, [], [])
        };
        ($ty: ty, $val: expr, $sol_ty: ty, $sol_trait: ty, $sol_val: expr, [$($ty_cvt: tt)*], [$($sol_ty_cvt: tt)*]) => {
            let encoded = <$ty as SolPrimitive>::encode(&$val);
            let encoded_alloy = <$sol_ty as $sol_trait>::abi_encode(&$sol_val);
            assert_eq!(encoded, encoded_alloy);

            let decoded = <$ty as SolPrimitive>::decode(&encoded);
            let decoded_alloy = <$sol_ty as $sol_trait>::abi_decode(&encoded, true);
            assert_eq!(decoded$($ty_cvt)*, decoded_alloy$($sol_ty_cvt)*);
        };
    }

    #[test]
    fn bool_works() {
        test_case!(bool, true);
        test_case!(bool, false);
    }

    #[test]
    fn signed_int_works() {
        test_case!(i8, -100);
        test_case!(i16, 10_000);
        test_case!(i32, -1_000_000);
        test_case!(i64, 1_000_000_000);
        test_case!(i128, -1_000_000_000_000);
    }

    #[test]
    fn unsigned_int_works() {
        // `SolValue` isn't implemented for `u8`.
        test_case!(u8, 100, Uint<8>, SolType);
        test_case!(u16, 10_000);
        test_case!(u32, 1_000_000);
        test_case!(i64, 1_000_000_000);
        test_case!(i128, 1_000_000_000_000);
    }

    #[test]
    fn string_works() {
        test_case!(String, String::from(""));
        test_case!(String, String::from("Hello, world!"));
    }

    #[test]
    fn fixed_bytes_works() {
        use alloy_sol_types::private::FixedBytes;

        macro_rules! fixed_bytes_test_case {
            ($($size: literal),+ $(,)*) => {
                $(
                    test_case!(
                        [u8; $size], [100u8; $size],
                        FixedBytes<$size>, SolValue, FixedBytes([100u8; $size]),
                        [.unwrap()], [.unwrap().0]
                    );
                )+
            };
        }

        fixed_bytes_test_case!(
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
            22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
        );
    }

    #[test]
    fn bytes_works() {
        use alloy_sol_types::private::Bytes;

        macro_rules! bytes_test_case {
            ($($fixture_size: literal),+ $(,)*) => {
                $(
                    let bytes = Vec::from([100u8; $fixture_size]);
                    let sol_bytes = Bytes::from(bytes.clone());

                    test_case!(
                        Vec<u8>, bytes,
                        Bytes, SolValue, sol_bytes,
                        [.unwrap().as_slice()], [.unwrap().as_ref()]
                    );
                )+
            };
        }

        // Number/size is the dynamic size of the `Vec`.
        bytes_test_case!(0, 1, 10, 20, 30, 40, 50, 60, 70);
    }

    #[test]
    fn fixed_array_works() {
        test_case!([bool; 2], [true, false]);

        test_case!([i8; 8], [100i8; 8]);
        test_case!([i16; 16], [-10_000i16; 16]);
        test_case!([i32; 32], [1_000_000i32; 32]);
        test_case!([i64; 64], [-1_000_000_000i64; 64]);
        test_case!([i128; 128], [1_000_000_000_000i128; 128]);

        test_case!([u16; 16], [10_000u16; 16]);
        test_case!([u32; 32], [1_000_000u32; 32]);
        test_case!([u64; 64], [1_000_000_000u64; 64]);
        test_case!([u128; 128], [1_000_000_000_000u128; 128]);

        test_case!(
            [String; 2],
            [String::from(""), String::from("Hello, world!")]
        );
    }

    #[test]
    fn dynamic_array_works() {
        test_case!(Vec<bool>, vec![true, false, false, true]);

        test_case!(Vec<i8>, Vec::from([100i8; 8]));
        test_case!(Vec<i16>, Vec::from([-10_000i16; 16]));
        test_case!(Vec<i32>, Vec::from([1_000_000i32; 32]));
        test_case!(Vec<i64>, Vec::from([-1_000_000_000i64; 64]));
        test_case!(Vec<i128>, Vec::from([1_000_000_000_000i128; 128]));

        test_case!(Vec<u8>, Vec::from([100u8; 8]));
        test_case!(Vec<u16>, Vec::from([10_000u16; 16]));
        test_case!(Vec<u32>, Vec::from([1_000_000u32; 32]));
        test_case!(Vec<u64>, Vec::from([1_000_000_000u64; 64]));
        test_case!(Vec<u128>, Vec::from([1_000_000_000_000u128; 128]));

        test_case!(
            Vec<String>,
            vec![String::from(""), String::from("Hello, world!")]
        );
    }

    #[test]
    fn tuple_works() {
        test_case!((), ());
        test_case!((bool,), (true,));
        // `SolValue` isn't implemented for `u8`.
        test_case!((u8,), (100u8,), (Uint<8>,), SolType);
        test_case!(
            (bool, i8, u32, String),
            (true, 100i8, 1_000_000u32, String::from("Hello, world!"))
        );
    }
}
