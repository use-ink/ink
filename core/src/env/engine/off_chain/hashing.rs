// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

use blake2_rfc;
use sha2::{
    Digest,
    Sha256,
};
use tiny_keccak::{
    Hasher,
    Keccak,
};
use twox_hash;

/// Conduct the BLAKE2 256-bit hash and place the result into `output`.
pub fn blake2_256_into(input: &[u8], output: &mut [u8; 32]) {
    output.copy_from_slice(blake2_rfc::blake2b::blake2b(32, &[], input).as_bytes());
}

/// Return the BLAKE2 256-bit hash for the given input.
pub fn blake2_256(input: &[u8]) -> [u8; 32] {
    let mut output = [0; 32];
    blake2_256_into(input, &mut output);
    output
}

/// Conduct the BLAKE2 128-bit hash and place the result into `output`.
pub fn blake2_128_into(input: &[u8], output: &mut [u8; 16]) {
    output.copy_from_slice(blake2_rfc::blake2b::blake2b(16, &[], input).as_bytes());
}

/// Return the BLAKE2 128-bit hash for the given input.
pub fn blake2_128(input: &[u8]) -> [u8; 16] {
    let mut output = [0; 16];
    blake2_128_into(input, &mut output);
    output
}

/// Conduct the TWOX (XX) 64-bit hash and place the result into `output`.
pub fn twox_64_into(input: &[u8], output: &mut [u8; 8]) {
    use ::core::hash::Hasher;
    let mut h0 = twox_hash::XxHash::with_seed(0);
    h0.write(input);
    let r0 = h0.finish();
    use byteorder::{
        ByteOrder,
        LittleEndian,
    };
    LittleEndian::write_u64(&mut output[0..8], r0);
}

/// Return the TWOX (XX) 64-bit hash for the given input.
pub fn twox_64(input: &[u8]) -> [u8; 8] {
    let mut output: [u8; 8] = [0; 8];
    twox_64_into(input, &mut output);
    output
}

/// Conduct the TWOX (XX) 128-bit hash and place the result into `output`.
pub fn twox_128_into(input: &[u8], output: &mut [u8; 16]) {
    use ::core::hash::Hasher;
    let mut h0 = twox_hash::XxHash::with_seed(0);
    let mut h1 = twox_hash::XxHash::with_seed(1);
    h0.write(input);
    h1.write(input);
    let r0 = h0.finish();
    let r1 = h1.finish();
    use byteorder::{
        ByteOrder,
        LittleEndian,
    };
    LittleEndian::write_u64(&mut output[0..8], r0);
    LittleEndian::write_u64(&mut output[8..16], r1);
}

/// Return the TWOX (XX) 128-bit hash for the given input.
pub fn twox_128(input: &[u8]) -> [u8; 16] {
    let mut output: [u8; 16] = [0; 16];
    twox_128_into(input, &mut output);
    output
}

/// Conduct the TWOX (XX) 256-bit hash and place the result into `output`.
pub fn twox_256_into(input: &[u8], output: &mut [u8; 32]) {
    use ::core::hash::Hasher;
    use byteorder::{
        ByteOrder,
        LittleEndian,
    };
    let mut h0 = twox_hash::XxHash::with_seed(0);
    let mut h1 = twox_hash::XxHash::with_seed(1);
    let mut h2 = twox_hash::XxHash::with_seed(2);
    let mut h3 = twox_hash::XxHash::with_seed(3);
    h0.write(input);
    h1.write(input);
    h2.write(input);
    h3.write(input);
    let r0 = h0.finish();
    let r1 = h1.finish();
    let r2 = h2.finish();
    let r3 = h3.finish();
    LittleEndian::write_u64(&mut output[0..8], r0);
    LittleEndian::write_u64(&mut output[8..16], r1);
    LittleEndian::write_u64(&mut output[16..24], r2);
    LittleEndian::write_u64(&mut output[24..32], r3);
}

/// Return the TWOX (XX) 256-bit hash for the given input.
pub fn twox_256(input: &[u8]) -> [u8; 32] {
    let mut output: [u8; 32] = [0; 32];
    twox_256_into(input, &mut output);
    output
}

/// Return the KECCAK 256-bit hash for the given input.
pub fn keccak_256(input: &[u8]) -> [u8; 32] {
    let mut keccak = Keccak::v256();
    keccak.update(input);
    let mut output = [0u8; 32];
    keccak.finalize(&mut output);
    output
}

/// Return the SHA2 256-bit hash for the given input.
pub fn sha2_256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.input(input);
    let mut output = [0u8; 32];
    output.copy_from_slice(&hasher.result());
    output
}
