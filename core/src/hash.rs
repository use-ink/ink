// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

//! Hashing utilities around the Keccak256 hasher.

use tiny_keccak;

use core::hash::{
    Hash,
    Hasher,
};

/// Keccak256 hasher.
#[derive(Clone)]
pub struct Keccak256Hasher {
    /// The internal keccak hasher.
    hasher: tiny_keccak::Keccak,
}

impl Default for Keccak256Hasher {
    fn default() -> Self {
        Keccak256Hasher {
            hasher: tiny_keccak::Keccak::new_keccak256(),
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
        arr[..8].clone_from_slice(&res[..8]);
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
