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

//! Provides type definitions and traits for the built-in cryptographic hashes.

/// The output type of built-in cryptographic hash functions.
pub trait HashOutput: private::Sealed {
    /// The output type of the crypto hash.
    ///
    /// This should be a byte array with some constant size such as `[u8; 32]`.
    type Type: Default;
}

/// Types that are usable as built-in cryptographic hashes.
pub trait CryptoHash: HashOutput + private::Sealed {
    /// Hashes the given raw byte input and copies the result into `output`.
    fn hash(input: &[u8], output: &mut <Self as HashOutput>::Type);
}

/// The SHA-2 crypto hash with 256-bit output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sha2x256 {}

/// The KECCAK crypto hash with 256-bit output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Keccak256 {}

/// The BLAKE-2 crypto hash with 256-bit output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Blake2x256 {}

/// The BLAKE-2 crypto hash with 128-bit output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Blake2x128 {}

mod private {
    /// Seals the implementation of `CryptoHash` and `HashOutput`.
    pub trait Sealed {}
}

impl private::Sealed for Sha2x256 {}
impl private::Sealed for Keccak256 {}
impl private::Sealed for Blake2x256 {}
impl private::Sealed for Blake2x128 {}

impl HashOutput for Sha2x256 {
    type Type = [u8; 32];
}

impl HashOutput for Keccak256 {
    type Type = [u8; 32];
}

impl HashOutput for Blake2x256 {
    type Type = [u8; 32];
}

impl HashOutput for Blake2x128 {
    type Type = [u8; 16];
}
