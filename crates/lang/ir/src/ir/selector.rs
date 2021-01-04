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

/// A function selector.
///
/// # Note
///
/// This is equal to the first four bytes of the SHA-3 hash of a function's name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Selector {
    bytes: [u8; 4],
}

impl Selector {
    /// Creates a new selector from the given bytes.
    pub fn new(bytes: [u8; 4]) -> Self {
        Self { bytes }
    }

    /// Returns the underlying four bytes.
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.bytes
    }

    /// Returns a unique identifier as `usize`.
    pub fn unique_id(self) -> usize {
        u32::from_le_bytes(self.bytes) as usize
    }
}

impl From<[u8; 4]> for Selector {
    fn from(bytes: [u8; 4]) -> Self {
        Self::new(bytes)
    }
}
