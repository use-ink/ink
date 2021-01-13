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

mod impls;

#[cfg(test)]
mod tests;

use crate::traits::KeyPtr;
use ink_env::hash::{
    Blake2x256,
    Keccak256,
    Sha2x256,
};
use ink_metadata::layout::{
    CryptoHasher,
    Layout,
};

/// Implemented by types that have a storage layout.
pub trait StorageLayout {
    /// Returns the static storage layout of `Self`.
    ///
    /// The given key pointer is guiding the allocation of static fields onto
    /// the contract storage regions.
    fn layout(key_ptr: &mut KeyPtr) -> Layout;
}

/// Types implementing this trait are supported layouting crypto hashers.
pub trait LayoutCryptoHasher {
    /// Returns the layout crypto hasher for `Self`.
    fn crypto_hasher() -> CryptoHasher;
}

impl LayoutCryptoHasher for Blake2x256 {
    fn crypto_hasher() -> CryptoHasher {
        CryptoHasher::Blake2x256
    }
}

impl LayoutCryptoHasher for Sha2x256 {
    fn crypto_hasher() -> CryptoHasher {
        CryptoHasher::Sha2x256
    }
}

impl LayoutCryptoHasher for Keccak256 {
    fn crypto_hasher() -> CryptoHasher {
        CryptoHasher::Keccak256
    }
}
