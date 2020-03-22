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

use self::hasher::FinishInto;
pub use self::hasher::{
    Blake2x128Hasher,
    Blake2x256Hasher,
    CryptoBuildHasher,
    CryptoHasher,
    InputBuffer,
    Keccakx256Hasher,
    Sha2x256Hasher,
    TwoxHasher,
    Wrap,
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
        docs(*_into):
        $( #[$doc_into:meta] )*
        docs(*):
        $( #[$doc:meta] )*
        fn $name:ident(struct $ty:ident($output_len:literal));
    ) => {
        paste::item! {
            $( #[$doc_into] )*
            pub fn [< $name _into >]<I, T>(input: &T, buffer: I, output: &mut [u8; $output_len])
            where
                T: Hash,
                I: InputBuffer,
            {
                let mut hasher = <$ty<_> as From<I>>::from(buffer);
                <T as Hash>::hash(&input, &mut hasher);
                <$ty<_> as FinishInto<[u8; $output_len]>>::finish_into(&hasher, output)
            }

            $( #[$doc] )*
            pub fn [< $name >]<I, T>(input: &T, buffer: I) -> [u8; $output_len]
            where
                T: Hash,
                I: InputBuffer,
            {
                let mut output = [0x00_u8; $output_len];
                [< $name _into >](input, buffer, &mut output);
                output
            }
        }
    };
}
impl_hash_fn_for! {
    docs(*_into):
    /// Conducts the SHA2 256-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Stores the resulting hash into the given output buffer.
    ///
    /// # Note
    ///
    /// Use [`sha2_256`] function if you do not need control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// In the following examples the `EXPECTED` buffer is defined as:
    ///
    /// ```
    /// const EXPECTED: [u8; 32] = [
    ///     123,  38, 102, 217, 176, 174, 109,  30, 100, 71, 85, 214, 203,
    ///     212,  16,  26,  88,   5, 194, 156, 138, 243, 34, 74,  67, 115,
    ///     178, 200, 65, 118, 14, 253
    /// ];
    /// ```
    ///
    /// ## 1. Using a new `Vec` as accumulating buffer.
    ///
    /// This is the simplest way to call this API but does not avoid heap
    /// allocations.
    ///
    /// ```
    /// # use ink_core::hash::sha2_256_into;
    /// # const EXPECTED: [u8; 32] = [
    /// #     123,  38, 102, 217, 176, 174, 109,  30, 100, 71, 85, 214, 203,
    /// #     212,  16,  26,  88,   5, 194, 156, 138, 243, 34, 74,  67, 115,
    /// #     178, 200, 65, 118, 14, 253
    /// # ];
    /// let hashable = (42, "foo", true); // Implements `core::hash::Hash`
    /// let mut output = [0x00_u8; 32]; // 256-bit buffer
    /// sha2_256_into(&hashable, Vec::new(), &mut output);
    /// assert_eq!(output, EXPECTED);
    /// ```
    ///
    /// ## 2. Using an existing `Vec` as accumulating buffer.
    ///
    /// This API is preferred if the call site already has an allocated buffer
    /// that it can reuse. This will reset the existing buffer and might grow it.
    ///
    /// ```
    /// # use ink_core::hash::sha2_256_into;
    /// # const EXPECTED: [u8; 32] = [
    /// #     123,  38, 102, 217, 176, 174, 109,  30, 100, 71, 85, 214, 203,
    /// #     212,  16,  26,  88,   5, 194, 156, 138, 243, 34, 74,  67, 115,
    /// #     178, 200, 65, 118, 14, 253
    /// # ];
    /// let hashable = (42, "foo", true); // Implements `core::hash::Hash`
    /// let mut output = [0x00_u8; 32]; // 256-bit buffer
    /// let mut buffer = Vec::with_capacity(32);
    /// sha2_256_into(&hashable, &mut buffer, &mut output);
    /// assert_eq!(output, EXPECTED);
    /// ```
    ///
    /// ## 3. Using a wrapped static buffer as accumulating buffer.
    ///
    /// This API avoids heap allocation completely but might panic in cases
    /// where the static buffer is too small.
    ///
    /// ```
    /// # use ink_core::hash::{sha2_256_into, Wrap};
    /// # const EXPECTED: [u8; 32] = [
    /// #     123,  38, 102, 217, 176, 174, 109,  30, 100, 71, 85, 214, 203,
    /// #     212,  16,  26,  88,   5, 194, 156, 138, 243, 34, 74,  67, 115,
    /// #     178, 200, 65, 118, 14, 253
    /// # ];
    /// let hashable = (42, "foo", true); // Implements `core::hash::Hash`
    /// let mut output = [0x00_u8; 32]; // 256-bit buffer
    /// let mut buffer = [0x00_u8; 64];
    /// sha2_256_into(&hashable, Wrap::from(buffer.as_mut()), &mut output);
    /// assert_eq!(output, EXPECTED);
    /// ```
    docs(*):
    /// Returns the SHA2 256-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Returns the resulting hash directly back to the caller.
    ///
    /// # Note
    ///
    /// Use [`sha2_256_into`] function if you need more control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// In the following examples the `EXPECTED` buffer is defined as:
    ///
    /// ```
    /// const EXPECTED: [u8; 32] = [
    ///     123,  38, 102, 217, 176, 174, 109,  30, 100, 71, 85, 214, 203,
    ///     212,  16,  26,  88,   5, 194, 156, 138, 243, 34, 74,  67, 115,
    ///     178, 200, 65, 118, 14, 253
    /// ];
    /// ```
    ///
    /// ## 1. Using a new `Vec` as accumulating buffer.
    ///
    /// This is the simplest way to call this API but does not avoid heap
    /// allocations.
    ///
    /// ```
    /// # use ink_core::hash::sha2_256;
    /// # const EXPECTED: [u8; 32] = [
    /// #     123,  38, 102, 217, 176, 174, 109,  30, 100, 71, 85, 214, 203,
    /// #     212,  16,  26,  88,   5, 194, 156, 138, 243, 34, 74,  67, 115,
    /// #     178, 200, 65, 118, 14, 253
    /// # ];
    /// assert_eq!(
    ///     sha2_256(&(42, "foo", true), Vec::new()),
    ///     EXPECTED,
    /// );
    /// ```
    ///
    /// ## 2. Using an existing `Vec` as accumulating buffer.
    ///
    /// This API is preferred if the call site already has an allocated buffer
    /// that it can reuse. This will reset the existing buffer and might grow it.
    ///
    /// ```
    /// # use ink_core::hash::sha2_256;
    /// # const EXPECTED: [u8; 32] = [
    /// #     123,  38, 102, 217, 176, 174, 109,  30, 100, 71, 85, 214, 203,
    /// #     212,  16,  26,  88,   5, 194, 156, 138, 243, 34, 74,  67, 115,
    /// #     178, 200, 65, 118, 14, 253
    /// # ];
    /// let mut buffer = Vec::with_capacity(32);
    /// assert_eq!(
    ///     sha2_256(&(42, "foo", true), &mut buffer),
    ///     EXPECTED,
    /// );
    /// ```
    ///
    /// ## 3. Using a wrapped static buffer as accumulating buffer.
    ///
    /// This API avoids heap allocation completely but might panic in cases
    /// where the static buffer is too small.
    ///
    /// ```
    /// # use ink_core::hash::{sha2_256, Wrap};
    /// # const EXPECTED: [u8; 32] = [
    /// #     123,  38, 102, 217, 176, 174, 109,  30, 100, 71, 85, 214, 203,
    /// #     212,  16,  26,  88,   5, 194, 156, 138, 243, 34, 74,  67, 115,
    /// #     178, 200, 65, 118, 14, 253
    /// # ];
    /// let mut buffer = [0x00_u8; 64];
    /// assert_eq!(
    ///     sha2_256(&(42, "foo", true), Wrap::from(buffer.as_mut())),
    ///     EXPECTED,
    /// );
    /// ```
    fn sha2_256(struct Sha2x256Hasher(32));
}
impl_hash_fn_for! {
    docs(*_into):
    /// Conducts the KECCAK 256-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Stores the resulting hash into the given output buffer.
    ///
    /// # Note
    ///
    /// Use [`keccak_256`] function if you do not need control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// The examples demonstrated in [`sha2_256_into`] can be reflected on this API.
    docs(*):
    /// Returns the KECCAK 256-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Returns the resulting hash directly back to the caller.
    ///
    /// # Note
    ///
    /// Use [`keccak_256_into`] function if you need more control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// The examples demonstrated in [`sha2_256`] can be reflected on this API.
    fn keccak_256(struct Keccakx256Hasher(32));
}
impl_hash_fn_for! {
    docs(*_into):
    /// Conducts the BLAKE2 256-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Stores the resulting hash into the given output buffer.
    ///
    /// # Note
    ///
    /// Use [`blake2_256`] function if you do not need control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// The examples demonstrated in [`sha2_256_into`] can be reflected on this API.
    docs(*):
    /// Returns the BLAKE2 256-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Returns the resulting hash directly back to the caller.
    ///
    /// # Note
    ///
    /// Use [`blake2_256_into`] function if you need more control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// The examples demonstrated in [`sha2_256`] can be reflected on this API.
    fn blake2_256(struct Blake2x256Hasher(32));
}
impl_hash_fn_for! {
    docs(*_into):
    /// Conducts the BLAKE2 128-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Stores the resulting hash into the given output buffer.
    ///
    /// # Note
    ///
    /// Use [`blake2_128`] function if you do not need control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// The examples demonstrated in [`sha2_256_into`] can be reflected on this API.
    docs(*):
    /// Returns the BLAKE2 128-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Returns the resulting hash directly back to the caller.
    ///
    /// # Note
    ///
    /// Use [`blake2_128_into`] function if you need more control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// The examples demonstrated in [`sha2_256`] can be reflected on this API.
    fn blake2_128(struct Blake2x128Hasher(16));
}
impl_hash_fn_for! {
    docs(*_into):
    /// Conducts the TWOX 256-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Stores the resulting hash into the given output buffer.
    ///
    /// # Note
    ///
    /// Use [`twox_256`] function if you do not need control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// The examples demonstrated in [`sha2_256_into`] can be reflected on this API.
    docs(*):
    /// Returns the TWOX 256-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Returns the resulting hash directly back to the caller.
    ///
    /// # Note
    ///
    /// Use [`twox_256_into`] function if you need more control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// The examples demonstrated in [`sha2_256`] can be reflected on this API.
    fn twox_256(struct TwoxHasher(32));
}
impl_hash_fn_for! {
    docs(*_into):
    /// Conducts the TWOX 128-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Stores the resulting hash into the given output buffer.
    ///
    /// # Note
    ///
    /// Use [`twox_128`] function if you do not need control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// The examples demonstrated in [`sha2_256_into`] can be reflected on this API.
    docs(*):
    /// Returns the TWOX 128-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Returns the resulting hash directly back to the caller.
    ///
    /// # Note
    ///
    /// Use [`twox_128_into`] function if you need more control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// The examples demonstrated in [`sha2_256`] can be reflected on this API.
    fn twox_128(struct TwoxHasher(16));
}
impl_hash_fn_for! {
    docs(*_into):
    /// Conducts the TWOX 64-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Stores the resulting hash into the given output buffer.
    ///
    /// # Note
    ///
    /// Use [`twox_64`] function if you do not need control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// The examples demonstrated in [`sha2_256_into`] can be reflected on this API.
    docs(*):
    /// Returns the TWOX 64-bit hash for the given hashable input.
    ///
    /// Uses the given buffer for accumulating the hash.
    /// Returns the resulting hash directly back to the caller.
    ///
    /// # Note
    ///
    /// Use [`twox_64_into`] function if you need more control over the `output` buffer.
    ///
    /// # Examples
    ///
    /// The examples demonstrated in [`sha2_256`] can be reflected on this API.
    fn twox_64(struct TwoxHasher(8));
}
