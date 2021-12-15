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

//! Implementations of supported cryptographic hash functions.

/// Helper routine implementing variable size BLAKE-2b hash computation.
fn blake2b_var(size: usize, input: &[u8], output: &mut [u8]) {
    use ::blake2::digest::{
        Update as _,
        VariableOutput as _,
    };

    let mut blake2 = blake2::Blake2bVar::new(size).expect(
        "The provided `size` is bigger than the maximum `OutputSize` for `Blake2Var` (32-bytes).",
    );
    blake2.update(input);
    blake2.finalize_variable(output).expect(
        "The provided `output` size does not match the hasher (`Blake2Var`) output size.",
    );
}

/// Conduct the BLAKE-2 256-bit hash and place the result into `output`.
pub fn blake2b_256(input: &[u8], output: &mut [u8; 32]) {
    blake2b_var(32, input, output)
}

/// Conduct the BLAKE-2 128-bit hash and place the result into `output`.
pub fn blake2b_128(input: &[u8], output: &mut [u8; 16]) {
    blake2b_var(16, input, output)
}

/// Conduct the KECCAK 256-bit hash and place the result into `output`.
pub fn keccak_256(input: &[u8], output: &mut [u8; 32]) {
    use ::sha3::{
        digest::generic_array::GenericArray,
        Digest as _,
    };
    let mut hasher = ::sha3::Keccak256::new();
    hasher.update(input);
    hasher.finalize_into(<&mut GenericArray<u8, _>>::from(&mut output[..]));
}

/// Conduct the SHA-2 256-bit hash and place the result into `output`.
pub fn sha2_256(input: &[u8], output: &mut [u8; 32]) {
    use ::sha2::{
        digest::generic_array::GenericArray,
        Digest as _,
    };
    let mut hasher = ::sha2::Sha256::new();
    hasher.update(input);
    hasher.finalize_into(<&mut GenericArray<u8, _>>::from(&mut output[..]));
}
