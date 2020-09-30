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

//! Supported cryptographic hashing algorithms.

/// Types that implement this trait are marker types that identify a supported
/// cryptographic hash function.
pub trait Hasher {
    /// The output of the hash function.
    ///
    /// # Note
    ///
    /// This is a byte slice with varying lengths, e.g. `[u8; 32]`, [`u8; 16]`, etc.
    type Output: Default;

    /// Finalizes the hash using the underlying procedure.
    fn finalize_immediate(input: &[u8], output: &mut Self::Output);
}

macro_rules! impl_hasher_for {
    (
        $( #[$doc:meta] )*
        struct $ty_name:ident($fn_name:ident, $output_len:literal);
    ) => {
        $( #[$doc] )*
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $ty_name {}

        impl Hasher for $ty_name {
            type Output = [u8; $output_len];

            fn finalize_immediate(input: &[u8], output: &mut Self::Output) {
                crate::env::hash_old::$fn_name(input, output)
            }
        }
    };
}
impl_hasher_for! {
    /// SHA2 256-bit hasher.
    struct Sha2x256Hasher(sha2_256, 32);
}
impl_hasher_for! {
    /// KECCAK 256-bit hasher.
    struct Keccak256Hasher(keccak_256, 32);
}
impl_hasher_for! {
    /// BLAKE2 256-bit hasher.
    struct Blake2x256Hasher(blake2_256, 32);
}
impl_hasher_for! {
    /// BLAKE2 128-bit hasher.
    struct Blake2x128Hasher(blake2_128, 16);
}
