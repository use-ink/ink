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
        Bytes as AlloyBytes,
        FixedBytes as AlloyFixedBytes,
    },
    sol_data,
    SolType as AlloySolType,
    SolValue,
};
use ink_prelude::{
    string::String,
    vec::Vec,
};
use primitive_types::{
    H256,
    U256,
};

use crate::{
    sol::{
        SolBytes,
        SolDecode,
        SolEncode,
        SolParamsDecode,
        SolParamsEncode,
        SolTypeDecode,
        SolTypeEncode,
    },
    types::{
        AccountId,
        Address,
        Hash,
    },
};

macro_rules! test_case_codec {
    ($ty: ty, $val: expr) => {
        test_case_codec!($ty, $val, $ty, alloy_sol_types::SolValue, $val, [], [])
    };
    ($ty: ty, $val: expr, $sol_ty: ty, $sol_trait: ty) => {
        test_case_codec!($ty, $val, $sol_ty, $sol_trait, $val, [], [])
    };
    ($ty: ty, $val: expr, $sol_ty: ty, $sol_trait: ty, $sol_val: expr, [$($ty_cvt: tt)*], [$($sol_ty_cvt: tt)*]) => {
        // `SolEncode` test.
        let encoded = <$ty as SolEncode>::encode(&$val);
        let encoded_alloy = <$sol_ty as $sol_trait>::abi_encode(&$sol_val);
        assert_eq!(encoded, encoded_alloy);

        // `SolDecode` test.
        let decoded = <$ty as SolDecode>::decode(&encoded);
        let decoded_alloy = <$sol_ty as $sol_trait>::abi_decode(&encoded);
        assert_eq!(decoded$($ty_cvt)*, decoded_alloy$($sol_ty_cvt)*);
    };
}

macro_rules! test_case {
    ($ty: ty, $val: expr) => {
        test_case!($ty, $val, $ty, alloy_sol_types::SolValue, $val, [], [])
    };
    ($ty: ty, $val: expr, $sol_ty: ty, $sol_trait: ty) => {
        test_case!($ty, $val, $sol_ty, $sol_trait, $val, [], [])
    };
    ($ty: ty, $val: expr, $sol_ty: ty, $sol_trait: ty, $sol_val: expr, [$($ty_cvt: tt)*], [$($sol_ty_cvt: tt)*]) => {
        // `SolTypeEncode` test.
        let encoded = <$ty as SolTypeEncode>::encode(&$val);
        let encoded_alloy = <$sol_ty as $sol_trait>::abi_encode(&$sol_val);
        assert_eq!(encoded, encoded_alloy);

        // `SolTypeDecode` test.
        let decoded = <$ty as SolTypeDecode>::decode(&encoded);
        let decoded_alloy = <$sol_ty as $sol_trait>::abi_decode(&encoded);
        assert_eq!(decoded$($ty_cvt)*, decoded_alloy$($sol_ty_cvt)*);

        // `SolEncode` and `SolDecode` test.
        test_case_codec!($ty, $val, $sol_ty, $sol_trait, $sol_val, [$($ty_cvt)*], [$($sol_ty_cvt)*]);
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
    // String
    test_case!(String, String::from(""));
    test_case!(String, String::from("Hello, world!"));

    // `Box<str>`
    test_case!(
        Box<str>,
        Box::from(""),
        String,
        SolValue,
        String::from(""),
        [.unwrap().as_ref()],
        [.unwrap().as_str()]
    );
    test_case!(
        Box<str>,
        Box::from("Hello, world!"),
        String,
        SolValue,
        String::from("Hello, world!"),
        [.unwrap().as_ref()],
        [.unwrap().as_str()]
    );
}

#[test]
fn address_works() {
    test_case!(
        Address, Address::from([1; 20]),
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
        [Address; 4], [Address::from([1; 20]); 4],
        [AlloyAddress; 4], SolValue, [AlloyAddress::from([1; 20]); 4],
        [.unwrap().map(|val| val.0)], [.unwrap().map(|val| val.0)]
    );
}

#[test]
fn dynamic_array_works() {
    test_case!(Vec<bool>, vec![true, false, false, true]);

    test_case!(
        Box<[bool]>,
        Box::from([true, false, false, true]),
        Vec<bool>,
        SolValue,
        vec![true, false, false, true],
        [.unwrap().as_ref()],
        [.unwrap().as_slice()]
    );

    test_case!(Vec<i8>, Vec::from([100i8; 8]));
    test_case!(Vec<i16>, Vec::from([-10_000i16; 16]));
    test_case!(Vec<i32>, Vec::from([1_000_000i32; 32]));
    test_case!(Vec<i64>, Vec::from([-1_000_000_000i64; 64]));
    test_case!(Vec<i128>, Vec::from([1_000_000_000_000i128; 128]));

    test_case!(
        Box<[i8]>,
        Box::from([100i8; 8]),
        Vec<i8>,
        SolValue,
        Vec::from([100i8; 8]),
        [.unwrap().as_ref()],
        [.unwrap().as_slice()]
    );

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
        Box<[String]>,
        Box::from([String::from(""), String::from("Hello, world!")]),
        Vec<String>,
        SolValue,
        vec![String::from(""), String::from("Hello, world!")],
        [.unwrap().as_ref()],
        [.unwrap().as_slice()]
    );

    test_case!(
        Vec<Address>, Vec::from([Address::from([1; 20]); 4]),
        Vec<AlloyAddress>, SolValue, Vec::from([AlloyAddress::from([1; 20]); 4]),
        [.unwrap().into_iter().map(|val| val.0).collect::<Vec<_>>()], [.unwrap().into_iter().map(|val| val.0).collect::<Vec<_>>()]
    );
}

#[test]
fn fixed_bytes_works() {
    test_case!(
        SolBytes<u8>, SolBytes(100u8),
        AlloyFixedBytes<1>, SolValue, AlloyFixedBytes([100u8; 1]),
        [.unwrap().0], [.unwrap().0[0]]
    );

    macro_rules! fixed_bytes_test_case {
        ($($size: literal),+ $(,)*) => {
            $(
                test_case!(
                    SolBytes<[u8; $size]>, SolBytes([100u8; $size]),
                    AlloyFixedBytes<$size>, SolValue, AlloyFixedBytes([100u8; $size]),
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
                let vec_bytes = SolBytes(data.clone());
                let sol_bytes = AlloyBytes::from(data);

                test_case!(
                    SolBytes<Vec<u8>>, vec_bytes,
                    AlloyBytes, SolValue, sol_bytes,
                    [.unwrap().as_slice()], [.unwrap().as_ref()]
                );

                let box_bytes = SolBytes(Box::from([100u8; $fixture_size]));
                test_case!(
                    SolBytes<Box<[u8]>>, box_bytes,
                    AlloyBytes, SolValue, sol_bytes,
                    [.unwrap().0.as_ref()], [.unwrap().as_ref()]
                );
            )+
        };
    }

    // Number/size is the dynamic size of the `Vec`.
    bytes_test_case!(0, 1, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100);
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
        ([Address; 4],), ([Address::from([1; 20]); 4],),
        ([AlloyAddress; 4],), SolValue, ([AlloyAddress::from([1; 20]); 4],),
        [.unwrap().0.map(|val| val.0)], [.unwrap().0.map(|val| val.0)]
    );
    test_case!(
        (Vec<Address>,), (Vec::from([Address::from([1; 20]); 4]),),
        (Vec<AlloyAddress>,), SolValue, (Vec::from([AlloyAddress::from([1; 20]); 4]),),
        [.unwrap().0.into_iter().map(|val| val.0).collect::<Vec<_>>()], [.unwrap().0.into_iter().map(|val| val.0).collect::<Vec<_>>()]
    );

    // fixed-size byte arrays.
    test_case!(
        (SolBytes<[u8; 32]>,),
        (SolBytes([100u8; 32]),),
        (AlloyFixedBytes<32>,),
        SolValue,
        (AlloyFixedBytes([100u8; 32]),),
        [.unwrap().0.0],
        [.unwrap().0.0]
    );

    // dynamic size byte arrays.
    test_case!(
        (SolBytes<Vec<u8>>,),
        (SolBytes(Vec::from([100u8; 64])),),
        (AlloyBytes,),
        SolValue,
        (AlloyBytes::from([100u8; 64]),),
        [.unwrap().0.0],
        [.unwrap().0.0]
    );
}

#[test]
fn account_id_works() {
    test_case_codec!(
        AccountId,
        AccountId([1; 32]),
        AlloyFixedBytes<32>,
        SolValue,
        AlloyFixedBytes([1; 32]),
        [.unwrap().0],
        [.unwrap().0]
    );
}

#[test]
fn hash_works() {
    test_case_codec!(
        Hash,
        Hash::from([1; 32]),
        AlloyFixedBytes<32>,
        SolValue,
        AlloyFixedBytes([1; 32]),
        [.unwrap().as_ref()],
        [.unwrap().0.as_slice()]
    );
}

#[test]
fn h256_works() {
    test_case_codec!(
        H256,
        H256([1; 32]),
        AlloyFixedBytes<32>,
        SolValue,
        AlloyFixedBytes([1; 32]),
        [.unwrap().0],
        [.unwrap().0]
    );
}

#[test]
fn custom_type_works() {
    // Example arbitrary type.
    struct MyType {
        size: i8,
        status: bool,
    }

    // `SolDecode` implementation/mapping.
    impl SolDecode for MyType {
        type SolType = (i8, bool);

        fn from_sol_type(value: Self::SolType) -> Self {
            Self {
                size: value.0,
                status: value.1,
            }
        }
    }

    // `SolEncode` implementation/mapping.
    impl<'a> SolEncode<'a> for MyType {
        // NOTE: Prefer reference based representation for better performance.
        type SolType = (&'a i8, &'a bool);

        fn to_sol_type(&'a self) -> Self::SolType {
            (&self.size, &self.status)
        }
    }

    impl MyType {
        fn into_tuple(self) -> (i8, bool) {
            (self.size, self.status)
        }
    }

    test_case_codec!(
        MyType,
        MyType { size: 100, status: true },
        (i8, bool),
        SolValue,
        (100i8, true),
        [.unwrap().into_tuple()],
        [.unwrap()]
    );
}

#[test]
fn encode_refs_works() {
    macro_rules! test_case_encode {
        ($ty: ty, $val: expr) => {
            test_case_encode!($ty, $val, $ty, alloy_sol_types::SolValue, $val, [], [])
        };
        ($ty: ty, $val: expr, $sol_ty: ty, $sol_trait: ty) => {
            test_case_encode!($ty, $val, $sol_ty, $sol_trait, $val, [], [])
        };
        ($ty: ty, $val: expr, $sol_ty: ty, $sol_trait: ty, $sol_val: expr, [$($ty_cvt: tt)*], [$($sol_ty_cvt: tt)*]) => {
            // `SolTypeEncode` test.
            let encoded = <$ty as SolTypeEncode>::encode(&$val);
            let encoded_alloy = <$sol_ty as $sol_trait>::abi_encode(&$sol_val);
            assert_eq!(encoded, encoded_alloy);

            // `SolEncode` test.
            let encoded = <$ty as SolEncode>::encode(&$val);
            let encoded_alloy = <$sol_ty as $sol_trait>::abi_encode(&$sol_val);
            assert_eq!(encoded, encoded_alloy);
        };
    }

    // bool
    test_case_encode!(&bool, &true, bool, SolValue, true, [], []);

    // integers
    test_case_encode!(&i8, &-100i8);
    test_case_encode!(&u128, &1_000_000_000_000u128);
    // U256
    use alloy_sol_types::private::U256 as AlloyU256;
    let value = 1_000_000_000_000_000u128;
    let bytes = value.to_be_bytes();
    test_case_encode!(
        &U256, &U256::from(value),
        AlloyU256, SolValue, AlloyU256::try_from_be_slice(bytes.as_slice()).unwrap(),
        [.unwrap().to_big_endian()], [.unwrap().to_be_bytes()]
    );

    // strings
    test_case_encode!(&str, "");
    test_case_encode!(&str, "Hello, world!");
    test_case_encode!(&String, &String::from("Hello, world!"));

    // address
    test_case_encode!(
        &Address, &Address::from([1; 20]),
        AlloyAddress, SolValue, AlloyAddress::from([1; 20]),
        [.unwrap().0], [.unwrap().0]
    );

    // array refs
    test_case_encode!(&[i8; 8], &[100i8; 8]);

    // slices
    test_case_encode!(&[i8], &[100i8; 8].as_slice());

    // fixed bytes refs
    test_case_encode!(
        &SolBytes<[u8; 32]>, &SolBytes([100u8; 32]),
        AlloyFixedBytes<32>, SolValue, AlloyFixedBytes([100u8; 32]),
        [.unwrap().0], [.unwrap().0]
    );

    // dynamic bytes refs
    let data = Vec::from([100u8; 64]);
    let bytes = SolBytes(data.clone());
    let sol_bytes = AlloyBytes::from(data);
    test_case_encode!(
        &SolBytes<Vec<u8>>, &bytes,
        AlloyBytes, SolValue, sol_bytes,
        [.unwrap().as_slice()], [.unwrap().as_ref()]
    );

    // tuple refs
    test_case_encode!(
        &(bool, i8, u32, String),
        &(true, 100i8, 1_000_000u32, String::from("Hello, world!")),
        (bool, i8, u32, String),
        SolValue,
        (true, 100i8, 1_000_000u32, String::from("Hello, world!")),
        [],
        []
    );

    // tuple of refs
    test_case_encode!(
        (&bool, &i8, &u32, &str),
        (&true, &100i8, &1_000_000u32, "Hello, world!"),
        (bool, i8, u32, String),
        SolValue,
        (true, 100i8, 1_000_000u32, String::from("Hello, world!")),
        [],
        []
    );
}

macro_rules! test_case_params {
    ($ty: ty, $val: expr) => {
        test_case_params!($ty, $val, $ty, alloy_sol_types::SolValue, $val, [], [])
    };
    ($ty: ty, $val: expr, $sol_ty: ty, $sol_trait: ty) => {
        test_case_params!($ty, $val, $sol_ty, $sol_trait, $val, [], [])
    };
    ($ty: ty, $val: expr, $sol_ty: ty, $sol_trait: ty, $sol_val: expr, [$($ty_cvt: tt)*], [$($sol_ty_cvt: tt)*]) => {
        // `SolParamsEncode` test.
        let encoded = <$ty as SolParamsEncode>::encode(&$val);
        let encoded_alloy = <$sol_ty as $sol_trait>::abi_encode_params(&$sol_val);
        assert_eq!(encoded, encoded_alloy);

        // `SolParamsDecode` test.
        let decoded = <$ty as SolParamsDecode>::decode(&encoded);
        let decoded_alloy = <$sol_ty as $sol_trait>::abi_decode_params(&encoded);
        assert_eq!(decoded$($ty_cvt)*, decoded_alloy$($sol_ty_cvt)*);
    };
}

#[test]
fn params_works() {
    test_case_params!((), ());
    test_case_params!((bool,), (true,));
    // `SolValue` isn't implemented for `u8`.
    test_case_params!((u8,), (100u8,), (sol_data::Uint<8>,), AlloySolType);
    test_case_params!(
        (bool, i8, u32, String),
        (true, 100i8, 1_000_000u32, String::from("Hello, world!"))
    );

    // simple sequences/collections.
    test_case_params!(([i8; 32],), ([100i8; 32],));
    test_case_params!((Vec<i8>,), (Vec::from([100i8; 64]),));
    test_case_params!(([i8; 32], Vec<i8>), ([100i8; 32], Vec::from([100i8; 64])));

    // sequences of addresses.
    test_case_params!(
        ([Address; 4],), ([Address::from([1; 20]); 4],),
        ([AlloyAddress; 4],), SolValue, ([AlloyAddress::from([1; 20]); 4],),
        [.unwrap().0.map(|val| val.0)], [.unwrap().0.map(|val| val.0)]
    );
    test_case_params!(
        (Vec<Address>,), (Vec::from([Address::from([1; 20]); 4]),),
        (Vec<AlloyAddress>,), SolValue, (Vec::from([AlloyAddress::from([1; 20]); 4]),),
        [.unwrap().0.into_iter().map(|val| val.0).collect::<Vec<_>>()], [.unwrap().0.into_iter().map(|val| val.0).collect::<Vec<_>>()]
    );

    // fixed-size byte arrays.
    test_case_params!(
        (SolBytes<[u8; 32]>,),
        (SolBytes([100u8; 32]),),
        (AlloyFixedBytes<32>,),
        SolValue,
        (AlloyFixedBytes([100u8; 32]),),
        [.unwrap().0.0],
        [.unwrap().0.0]
    );

    // dynamic size byte arrays.
    test_case_params!(
        (SolBytes<Vec<u8>>,),
        (SolBytes(Vec::from([100u8; 64])),),
        (AlloyBytes,),
        SolValue,
        (AlloyBytes::from([100u8; 64]),),
        [.unwrap().0.0],
        [.unwrap().0.0]
    );
}
