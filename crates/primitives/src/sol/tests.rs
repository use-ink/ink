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

//! Sanity checks to ensure our `SolTypeDecode`, `SolTypeEncode`, `SolDecode` and
//! `SolEncode` implementations match alloy's `SolValue` equivalents.

use alloy_sol_types::{
    private::{
        Address as AlloyAddress,
        Bytes as SolBytes,
        FixedBytes as SolFixedBytes,
    },
    sol_data,
    SolType as AlloySolType,
    SolValue,
};
use ink_prelude::{
    string::String,
    vec::Vec,
};
use primitive_types::U256;

use crate::{
    sol::{
        AsSolBytes,
        SolDecode,
        SolEncode,
        SolTypeDecode,
        SolTypeEncode,
    },
    types::{
        AccountId,
        Address,
        Hash,
    },
};

macro_rules! test_case {
    ($ty: ty, $val: expr) => {
        test_case!($ty, $val, $ty, alloy_sol_types::SolValue, $val, [], [])
    };
    ($ty: ty, $val: expr, $sol_ty: ty, $sol_trait: ty) => {
        test_case!($ty, $val, $sol_ty, $sol_trait, $val, [], [])
    };
    ($ty: ty, $val: expr, $sol_ty: ty, $sol_trait: ty, $sol_val: expr, [$($ty_cvt: tt)*], [$($sol_ty_cvt: tt)*]) => {
        let encoded = <$ty as SolTypeEncode>::encode(&$val);
        let encoded_codec = <$ty as SolEncode>::encode(&$val);
        let encoded_alloy = <$sol_ty as $sol_trait>::abi_encode(&$sol_val);
        assert_eq!(encoded, encoded_alloy);
        assert_eq!(encoded_codec, encoded_alloy);

        let decoded = <$ty as SolTypeDecode>::decode(&encoded);
        let decoded_codec = <$ty as SolDecode>::decode(&encoded);
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
    test_case!(u8, 100, sol_data::Uint<8>, AlloySolType);
    test_case!(u16, 10_000);
    test_case!(u32, 1_000_000);
    test_case!(i64, 1_000_000_000);
    test_case!(i128, 1_000_000_000_000);

    // U256
    use alloy_sol_types::private::U256 as AlloyU256;
    let value = 1_000_000_000_000_000u128;
    let bytes = value.to_be_bytes();
    test_case!(
        U256, U256::from(value),
        AlloyU256, SolValue, AlloyU256::try_from_be_slice(bytes.as_slice()).unwrap(),
        [.unwrap().to_big_endian()], [.unwrap().to_be_bytes()]
    );
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
        sol_data::FixedArray<sol_data::Uint<8>, 8>,
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
        sol_data::Array<sol_data::Uint<8>>,
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
                        AsSolBytes<[u8; $size]>, AsSolBytes([100u8; $size]),
                        SolFixedBytes<$size>, SolValue, SolFixedBytes([100u8; $size]),
                        [.unwrap().0], [.unwrap().0]
                    );
                )+
            };
        }

    fixed_bytes_test_case!(
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
        23, 24, 25, 26, 27, 28, 29, 30, 31, 32
    );
}

#[test]
fn bytes_works() {
    macro_rules! bytes_test_case {
            ($($fixture_size: literal),+ $(,)*) => {
                $(
                    let data = Vec::from([100u8; $fixture_size]);
                    let bytes = AsSolBytes(data.clone());
                    let sol_bytes = SolBytes::from(data);

                    test_case!(
                        AsSolBytes<Vec<u8>>, bytes,
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
    test_case!((u8,), (100u8,), (sol_data::Uint<8>,), AlloySolType);
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
        (AsSolBytes<[u8; 32]>,),
        (AsSolBytes([100u8; 32]),),
        (SolFixedBytes<32>,),
        SolValue,
        (SolFixedBytes([100u8; 32]),),
        [.unwrap().0.0],
        [.unwrap().0.0]
    );

    // dynamic size byte arrays.
    test_case!(
        (AsSolBytes<Vec<u8>>,),
        (AsSolBytes(Vec::from([100u8; 64])),),
        (SolBytes,),
        SolValue,
        (SolBytes::from([100u8; 64]),),
        [.unwrap().0.0],
        [.unwrap().0.0]
    );
}

#[test]
fn account_id_works() {
    let account_id = AccountId([1; 32]);
    let bytes = SolFixedBytes([1; 32]);

    let encoded = <AccountId as SolEncode>::encode(&account_id);
    let encoded_alloy = <SolFixedBytes<32> as SolValue>::abi_encode(&bytes);
    assert_eq!(encoded, encoded_alloy);

    let decoded = <AccountId as SolDecode>::decode(&encoded);
    let decoded_alloy = <SolFixedBytes<32> as SolValue>::abi_decode(&encoded, true);
    assert_eq!(decoded.unwrap().0, decoded_alloy.unwrap().0);
}

#[test]
fn hash_works() {
    let hash = Hash::from([1; 32]);
    let bytes = SolFixedBytes([1; 32]);

    let encoded = <Hash as SolEncode>::encode(&hash);
    let encoded_alloy = <SolFixedBytes<32> as SolValue>::abi_encode(&bytes);
    assert_eq!(encoded, encoded_alloy);

    let decoded = <Hash as SolDecode>::decode(&encoded);
    let decoded_alloy = <SolFixedBytes<32> as SolValue>::abi_decode(&encoded, true);
    assert_eq!(
        decoded.unwrap().as_ref(),
        decoded_alloy.unwrap().0.as_slice()
    );
}
