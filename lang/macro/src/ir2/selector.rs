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

use proc_macro2::Ident;

/// A function selector.
///
/// # Note
///
/// This is equal to the first four bytes of the SHA-3 hash of a function's name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Selector([u8; 4]);

impl Selector {
    /// Creates a new selector from the given bytes.
    pub fn new(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }

    /// Returns the underlying four bytes.
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }

    /// Returns a unique identifier as `usize`.
    pub fn unique_id(self) -> usize {
        u32::from_le_bytes(self.0) as usize
    }
}

impl From<&'_ Ident> for Selector {
    fn from(ident: &Ident) -> Self {
        Self::from(ident.to_string().as_str())
    }
}

impl From<&'_ str> for Selector {
    fn from(name: &str) -> Self {
        let sha3_hash = ink_primitives::hash::keccak256(name.as_bytes());
        Self([sha3_hash[0], sha3_hash[1], sha3_hash[2], sha3_hash[3]])
    }
}
