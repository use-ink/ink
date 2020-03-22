// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

//! High-level, built-in and efficient cryptographic hashing routines.

mod hasher;

use core::hash::Hash;

use self::hasher::{FinishInto};
pub use self::hasher::{
    Wrap,
    Blake2x128Hasher,
    Blake2x256Hasher,
    CryptoBuildHasher,
    CryptoHasher,
    InputBuffer,
    Keccakx256Hasher,
    Sha2x256Hasher,
    TwoxHasher,
};

#[doc(inline)]
pub use crate::env::hash::{
    blake2_128 as blake2_128_raw_into,
    blake2_256 as blake2_256_raw_into,
    keccak_256 as keccak_256_raw_into,
    sha2_256 as sha2_256_raw_into,
    twox_128 as twox_128_raw_into,
    twox_256 as twox_256_raw_into,
    twox_64 as twox_64_raw_into,
};

macro_rules! impl_hash_fn_raw {
    (
        $(
            $( #[$doc:meta] )*
            fn $name:ident($output_len:literal);
        )*
    ) => {
        $(
            paste::item! {
                $( #[$doc] )*
                pub fn $name(input: &[u8]) -> [u8; $output_len] {
                    let mut output = [0x00_u8; $output_len];
                    self::[<$name _into>](input, &mut output);
                    output
                }
            }
        )*
    };
}
impl_hash_fn_raw! {
    /// Returns the SHA2 256-bit hash for the given input.
    fn sha2_256_raw(32);
    /// Returns the KECCAK 256-bit hash for the given input.
    fn keccak_256_raw(32);
    /// Returns the BLAKE2 256-bit hash for the given input.
    fn blake2_256_raw(32);
    /// Returns the BLAKE2 128-bit hash for the given input.
    fn blake2_128_raw(16);
    /// Returns the TWOX 256-bit hash for the given input.
    fn twox_256_raw(32);
    /// Returns the TWOX 128-bit hash for the given input.
    fn twox_128_raw(16);
    /// Returns the TWOX 64-bit hash for the given input.
    fn twox_64_raw(8);
}

macro_rules! impl_hash_fn_for {
    (
        $( #[$doc:meta] )*
        fn $name:ident(struct $ty:ident($output_len:literal));
    ) => {
        paste::item! {
            $( #[$doc] )*
            ///
            /// Uses the given input buffer for accumulating the hash.
            /// Stores the resulting hash into the given output buffer.
            pub fn [< $name _into_using >]<I, T>(value: &T, input: I, output: &mut [u8; $output_len])
            where
                T: Hash,
                I: InputBuffer,
            {
                let mut hasher = <$ty<_> as From<I>>::from(input);
                <T as Hash>::hash(&value, &mut hasher);
                <$ty<_> as FinishInto<[u8; $output_len]>>::finish_into(&hasher, output)
            }

            $( #[$doc] )*
            ///
            /// Uses the given input buffer for accumulating the hash.
            /// Returns the resulting hash.
            pub fn [< $name _using >]<I, T>(value: &T, input: I) -> [u8; $output_len]
            where
                T: Hash,
                I: InputBuffer,
            {
                let mut output = [0x00_u8; $output_len];
                [< $name _into_using >](value, input, &mut output);
                output
            }
        }
    };
}
impl_hash_fn_for! {
    /// Conducts the SHA2 256-bit hash for the given hashable value.
    fn sha2_256(struct Sha2x256Hasher(32));
}
impl_hash_fn_for! {
    /// Conducts the KECCAK 256-bit hash for the given hashable value.
    fn keccak_256(struct Keccakx256Hasher(32));
}
impl_hash_fn_for! {
    /// Conducts the BLAKE2 256-bit hash for the given hashable value.
    fn blake2_256(struct Blake2x256Hasher(32));
}
impl_hash_fn_for! {
    /// Conducts the BLAKE2 128-bit hash for the given hashable value.
    fn blake2_128(struct Blake2x128Hasher(16));
}
impl_hash_fn_for! {
    /// Conducts the TWOX 256-bit hash for the given hashable value.
    fn twox_256(struct TwoxHasher(32));
}
impl_hash_fn_for! {
    /// Conducts the TWOX 128-bit hash for the given hashable value.
    fn twox_128(struct TwoxHasher(16));
}
impl_hash_fn_for! {
    /// Conducts the TWOX 64-bit hash for the given hashable value.
    fn twox_64(struct TwoxHasher(8));
}
