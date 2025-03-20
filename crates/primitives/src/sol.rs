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

use alloy_sol_types::{
    private::SolTypeValue,
    sol_data,
    SolType as AlloySolType,
};
use core::borrow::Borrow;
use impl_trait_for_tuples::impl_for_tuples;
use paste::paste;

use crate::types::Address;
use from::SolFrom;

pub use bytes::AsBytes;

/// Maps an arbitrary Rust type to a Solidity representation for Solidity ABI
/// encoding/decoding.
pub trait SolCodec: From<Self::SolType> + Borrow<Self::SolType> {
    /// Equivalent Solidity type.
    type SolType: SolType;

    /// Name of equivalent Solidity type.
    const SOL_NAME: &'static str =
        <<Self::SolType as SolType>::AlloyType as AlloySolType>::SOL_NAME;

    /// Solidity ABI encode the value.
    fn encode(&self) -> Vec<u8> {
        <Self::SolType as SolType>::encode(self.borrow())
    }

    /// Solidity ABI decode into this type.
    fn decode(data: &[u8]) -> Result<Self, alloy_sol_types::Error> {
        <Self::SolType as SolType>::decode(data).map(Self::from)
    }
}

impl<T: SolType> SolCodec for T {
    type SolType = T;
}

/// A Rust equivalent of a Solidity type that implements logic for Solidity ABI
/// encoding/decoding.
///
/// | Rust/ink! type | Solidity type | Notes |
/// | -------------- | ------------- | ----- |
/// | `bool` | `bool` ||
/// | `iN` for `N ∈ {8,16,32,64,128}` | `intN` | e.g `i8` <=> `int8` |
/// | `uN` for `N ∈ {8,16,32,64,128}` | `uintN` | e.g `u8` <=> `uint8` |
/// | `String` | `string` ||
/// | `Address` | `address` ||
/// | `[T; N]` for `const N: usize` | `T[N]` | e.g. `[i8; 64]` <=> `int8[64]` |
/// | `Vec<T>` | `T[]` | e.g. `Vec<i8>` <=> `int8[]` |
/// | `AsBytes<[u8; N]>` for `1 <= N <= 32` |  `bytesN` | e.g. `AsBytes<[u8; 1]>` <=> `bytes1` |
/// | `AsBytes<Vec<u8>>` |  `bytes` ||
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
    /// Equivalent Solidity type from [`alloy_sol_types`].
    type AlloyType: AlloySolType;

    /// Solidity ABI encode the value.
    fn encode(&self) -> Vec<u8> {
        <Self::AlloyType as AlloySolType>::abi_encode(self)
    }

    /// Solidity ABI decode into this type.
    fn decode(data: &[u8]) -> Result<Self, alloy_sol_types::Error> {
        <Self::AlloyType as AlloySolType>::abi_decode(data, false).map(Self::sol_from)
    }
}

macro_rules! impl_primitive {
    ($($ty: ty => $sol_ty: ty),+ $(,)*) => {
        $(
            impl SolType for $ty {
                type AlloyType = $sol_ty;
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
    [T: SolType] [T] => sol_data::Array<T::AlloyType> [],
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

mod private {
    /// Seals implementations of `SolType`.
    pub trait Sealed {}
}

/// Sanity checks to ensure our `SolType` implementations match alloy's `SolValue`
/// equivalents.
#[cfg(test)]
mod tests {
    use super::*;
    use alloy_sol_types::{
        private::{
            Address as AlloyAddress,
            Bytes as SolBytes,
            FixedBytes as SolFixedBytes,
        },
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
            let encoded = <$ty as SolType>::encode(&$val);
            let encoded_codec = <$ty as SolCodec>::encode(&$val);
            let encoded_alloy = <$sol_ty as $sol_trait>::abi_encode(&$sol_val);
            assert_eq!(encoded, encoded_alloy);
            assert_eq!(encoded_codec, encoded_alloy);

            let decoded = <$ty as SolType>::decode(&encoded);
            let decoded_codec = <$ty as SolCodec>::decode(&encoded);
            let decoded_alloy = <$sol_ty as $sol_trait>::abi_decode(&encoded, true);
            assert_eq!(decoded$($ty_cvt)*, decoded_alloy.clone()$($sol_ty_cvt)*);
            assert_eq!(decoded_codec$($ty_cvt)*, decoded_alloy$($sol_ty_cvt)*);
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
        test_case!(u8, 100, Uint<8>, AlloySolType);
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
    fn address_works() {
        test_case!(
            Address, Address([1; 20]),
            AlloyAddress, SolValue, AlloyAddress::from([1; 20]),
            [.unwrap().0], [.unwrap().0]
        );
    }

    #[test]
    fn fixed_array_works() {
        test_case!([bool; 2], [true, false]);

        test_case!([i8; 8], [100i8; 8]);
        test_case!([i16; 16], [-10_000i16; 16]);
        test_case!([i32; 32], [1_000_000i32; 32]);
        test_case!([i64; 64], [-1_000_000_000i64; 64]);
        test_case!([i128; 128], [1_000_000_000_000i128; 128]);

        // `SolValue` for `[u8; N]` maps to `bytesN` for `1 <= N <= 32`.
        test_case!(
            [u8; 8],
            [100u8; 8],
            sol_data::FixedArray<Uint<8>, 8>,
            AlloySolType
        );
        test_case!([u16; 16], [10_000u16; 16]);
        test_case!([u32; 32], [1_000_000u32; 32]);
        test_case!([u64; 64], [1_000_000_000u64; 64]);
        test_case!([u128; 128], [1_000_000_000_000u128; 128]);

        test_case!(
            [String; 2],
            [String::from(""), String::from("Hello, world!")]
        );

        test_case!(
            [Address; 4], [Address([1; 20]); 4],
            [AlloyAddress; 4], SolValue, [AlloyAddress::from([1; 20]); 4],
            [.unwrap().map(|val| val.0)], [.unwrap().map(|val| val.0)]
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

        // `SolValue` for `Vec<u8>` maps to `bytes`.
        test_case!(
            Vec<u8>,
            Vec::from([100u8; 8]),
            sol_data::Array<Uint<8>>,
            AlloySolType
        );
        test_case!(Vec<u16>, Vec::from([10_000u16; 16]));
        test_case!(Vec<u32>, Vec::from([1_000_000u32; 32]));
        test_case!(Vec<u64>, Vec::from([1_000_000_000u64; 64]));
        test_case!(Vec<u128>, Vec::from([1_000_000_000_000u128; 128]));

        test_case!(
            Vec<String>,
            vec![String::from(""), String::from("Hello, world!")]
        );

        test_case!(
            Vec<Address>, Vec::from([Address([1; 20]); 4]),
            Vec<AlloyAddress>, SolValue, Vec::from([AlloyAddress::from([1; 20]); 4]),
            [.unwrap().into_iter().map(|val| val.0).collect::<Vec<_>>()], [.unwrap().into_iter().map(|val| val.0).collect::<Vec<_>>()]
        );
    }

    #[test]
    fn fixed_bytes_works() {
        macro_rules! fixed_bytes_test_case {
            ($($size: literal),+ $(,)*) => {
                $(
                    test_case!(
                        AsBytes<[u8; $size]>, AsBytes([100u8; $size]),
                        SolFixedBytes<$size>, SolValue, SolFixedBytes([100u8; $size]),
                        [.unwrap().0], [.unwrap().0]
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
        macro_rules! bytes_test_case {
            ($($fixture_size: literal),+ $(,)*) => {
                $(
                    let bytes = AsBytes(Vec::from([100u8; $fixture_size]));
                    let sol_bytes = SolBytes::from(bytes.clone());

                    test_case!(
                        AsBytes<Vec<u8>>, bytes,
                        SolBytes, SolValue, sol_bytes,
                        [.unwrap().as_slice()], [.unwrap().as_ref()]
                    );
                )+
            };
        }

        // Number/size is the dynamic size of the `Vec`.
        bytes_test_case!(0, 1, 10, 20, 30, 40, 50, 60, 70);
    }

    #[test]
    fn tuple_works() {
        test_case!((), ());
        test_case!((bool,), (true,));
        // `SolValue` isn't implemented for `u8`.
        test_case!((u8,), (100u8,), (Uint<8>,), AlloySolType);
        test_case!(
            (bool, i8, u32, String),
            (true, 100i8, 1_000_000u32, String::from("Hello, world!"))
        );

        // simple sequences/collections.
        test_case!(([i8; 32],), ([100i8; 32],));
        test_case!((Vec<i8>,), (Vec::from([100i8; 64]),));
        test_case!(([i8; 32], Vec<i8>), ([100i8; 32], Vec::from([100i8; 64])));

        // sequences of addresses.
        test_case!(
            ([Address; 4],), ([Address([1; 20]); 4],),
            ([AlloyAddress; 4],), SolValue, ([AlloyAddress::from([1; 20]); 4],),
            [.unwrap().0.map(|val| val.0)], [.unwrap().0.map(|val| val.0)]
        );
        test_case!(
            (Vec<Address>,), (Vec::from([Address([1; 20]); 4]),),
            (Vec<AlloyAddress>,), SolValue, (Vec::from([AlloyAddress::from([1; 20]); 4]),),
            [.unwrap().0.into_iter().map(|val| val.0).collect::<Vec<_>>()], [.unwrap().0.into_iter().map(|val| val.0).collect::<Vec<_>>()]
        );

        // fixed-size byte arrays.
        test_case!(
            (AsBytes<[u8; 32]>,),
            (AsBytes([100u8; 32]),),
            (SolFixedBytes<32>,),
            SolValue,
            (SolFixedBytes([100u8; 32]),),
            [.unwrap().0.0],
            [.unwrap().0.0]
        );

        // dynamic size byte arrays.
        test_case!(
            (AsBytes<Vec<u8>>,),
            (AsBytes(Vec::from([100u8; 64])),),
            (SolBytes,),
            SolValue,
            (SolBytes::from([100u8; 64]),),
            [.unwrap().0.0],
            [.unwrap().0.0]
        );
    }
}
