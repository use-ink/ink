// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

// use super::*;
use super::Key2 as Key;

const TEST_BYTES: [u8; 32] = *b"\
        \x00\x01\x02\x03\x04\x05\x06\x07\
        \x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\
        \x10\x11\x12\x13\x14\x15\x16\x17\
        \x18\x19\x1A\x1B\x1C\x1D\x1E\x1F\
    ";

mod key {
    use super::*;
    use core::ops::AddAssign;
    use scale::{
        Decode,
        Encode,
    };

    #[test]
    fn default_works() {
        let mut default_key = <Key as Default>::default();
        assert_eq!(default_key, Key::from([0x00_u8; 32]));
        assert_eq!(default_key.as_ref(), &[0x00_u8; 32]);
        assert_eq!(default_key.as_mut(), &mut [0x00_u8; 32]);
    }

    #[test]
    fn debug_works() {
        let key = Key::from(TEST_BYTES);
        assert_eq!(
            format!("{:?}", key),
            String::from(
                "Key(0x\
                    _00010203_04050607\
                    _08090A0B_0C0D0E0F\
                    _10111213_14151617\
                    _18191A1B_1C1D1E1F\
                )"
            ),
        );
    }

    #[test]
    fn display_works() {
        let key = Key::from(TEST_BYTES);
        assert_eq!(
            format!("{}", key),
            String::from(
                "0x\
                    _00010203_04050607\
                    _08090A0B_0C0D0E0F\
                    _10111213_14151617\
                    _18191A1B_1C1D1E1F"
            ),
        );
    }

    #[test]
    fn from_works() {
        let mut bytes = TEST_BYTES;
        assert_eq!(Key::from(TEST_BYTES).as_ref(), &bytes);
        assert_eq!(Key::from(TEST_BYTES).as_mut(), &mut bytes);
    }

    #[test]
    fn encode_decode_works() {
        let key = Key::from(TEST_BYTES);
        let encoded = key.encode();
        let decoded = Key::decode(&mut &encoded[..]).unwrap();
        assert_eq!(key, decoded);
    }

    #[test]
    fn encode_works() {
        let bytes = TEST_BYTES;
        let encoded = Key::from(bytes).encode();
        assert_eq!(encoded, bytes);
    }

    #[test]
    fn decode_works() {
        let bytes = TEST_BYTES;
        let decoded = Key::decode(&mut &bytes[..]).unwrap();
        assert_eq!(decoded, Key::from(bytes));
    }

    #[test]
    fn codec_hints_work() {
        let key = Key::default();
        assert_eq!(key.size_hint(), 32);
        assert_eq!(key.encoded_size(), 32);
        assert_eq!(Key::encoded_fixed_size(), Some(32));
    }

    #[test]
    fn add_one_to_zero_works() {
        let bytes = [0x00; 32];
        let expected = {
            let mut bytes = [0x00; 32];
            bytes[0] = 0x01;
            bytes
        };
        let mut key = Key::from(bytes);
        key.add_assign(1u64);
        assert_eq!(key.as_ref(), &expected);
    }

    #[test]
    fn add_with_ovfl_works() {
        const BYTES: [u8; 32] = *b"\
            \xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\
            \x00\x00\x00\x00\x00\x00\x00\x00\
            \x00\x00\x00\x00\x00\x00\x00\x00\
            \x00\x00\x00\x00\x00\x00\x00\x00\
        ";
        let expected = {
            let mut expected = [0x00; 32];
            expected[8] = 0x01;
            expected
        };
        let mut key = Key::from(BYTES);
        key.add_assign(1u64);
        assert_eq!(key.as_ref(), &expected);
    }

    #[test]
    fn add_with_ovfl_2_works() {
        const BYTES: [u8; 32] = *b"\
            \xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\
            \xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\
            \x00\x00\x00\x00\x00\x00\x00\x00\
            \x00\x00\x00\x00\x00\x00\x00\x00\
        ";
        let expected = {
            let mut expected = [0x00; 32];
            expected[16] = 0x01;
            expected
        };
        let mut key = Key::from(BYTES);
        key.add_assign(1u64);
        assert_eq!(key.as_ref(), &expected);
    }

    #[test]
    fn add_with_ovfl_3_works() {
        const BYTES: [u8; 32] = *b"\
            \xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\
            \xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\
            \xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\
            \x00\x00\x00\x00\x00\x00\x00\x00\
        ";
        let expected = {
            let mut expected = [0x00; 32];
            expected[24] = 0x01;
            expected
        };
        let mut key = Key::from(BYTES);
        key.add_assign(1u64);
        assert_eq!(key.as_ref(), &expected);
    }

    #[test]
    fn add_with_wrap_works() {
        const BYTES: [u8; 32] = [0xFF; 32];
        let expected = [0x00; 32];
        let mut key = Key::from(BYTES);
        key.add_assign(1u64);
        assert_eq!(key.as_ref(), &expected);
    }

    #[test]
    fn add_to_zero_works() {
        const TEST_VALUES: &[u64] = &[0, 1, 42, 10_000, u32::MAX as u64, u64::MAX];
        for test_value in TEST_VALUES {
            let mut key = <Key as Default>::default();
            let expected = {
                let mut expected = [0x00; 32];
                expected[0..8].copy_from_slice(&test_value.to_le_bytes());
                expected
            };
            key.add_assign(*test_value);
            assert_eq!(key.as_ref(), &expected);
        }
    }
}
