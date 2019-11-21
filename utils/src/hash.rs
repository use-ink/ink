// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

//! Hashing utilities around the Keccak256 hasher.

use core::hash::{
    Hash,
    Hasher,
};

use tiny_keccak::Hasher as TinyKeccakHasher;

/// Keccak256 hasher.
#[derive(Clone)]
pub struct Keccak256Hasher {
    /// The internal keccak hasher.
    hasher: tiny_keccak::Keccak,
}

impl Default for Keccak256Hasher {
    fn default() -> Self {
        Keccak256Hasher {
            hasher: tiny_keccak::Keccak::v256(),
        }
    }
}

impl Keccak256Hasher {
    /// Returns the hash value for the values written so far.
    ///
    /// If you need to start a fresh hash value, you will have to create a new hasher.
    pub fn finish256(self) -> [u8; 32] {
        let mut res = [0; 32];
        self.hasher.finalize(&mut res);
        res
    }

    pub fn finish64(self) -> [u8; 8] {
        let mut arr = [0; 8];
        let res = self.finish256();
        arr[..8].copy_from_slice(&res[..8]);
        arr
    }
}

fn bytes8_to_u64(bytes: [u8; 8]) -> u64 {
    bytes
        .iter()
        .enumerate()
        .map(|(n, &ext)| u64::from(ext) << (n * 8))
        .fold(0u64, |acc, ext| acc | ext)
}

impl Hasher for Keccak256Hasher {
    /// Returns the hash value for the values written so far.
    ///
    /// If you need to start a fresh hash value, you will have to create a new hasher.
    fn finish(&self) -> u64 {
        bytes8_to_u64(self.clone().finish64())
    }

    /// Writes some data into the hasher.
    fn write(&mut self, bytes: &[u8]) {
        self.hasher.update(bytes)
    }
}

/// Returns the keccak-256 hash for the given byte slice.
pub fn keccak256<T>(val: &T) -> [u8; 32]
where
    T: ?Sized + Hash,
{
    let mut hasher = Keccak256Hasher::default();
    val.hash(&mut hasher);
    hasher.finish256()
}
