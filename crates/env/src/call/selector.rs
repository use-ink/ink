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

use derive_more::From;

/// The function selector.
#[derive(Debug, Copy, Clone, PartialEq, Eq, From, scale::Decode, scale::Encode)]
pub struct Selector {
    /// The 4 underlying bytes.
    bytes: [u8; 4],
}

impl Selector {
    /// Creates a selector directly from 4 bytes.
    pub const fn new(bytes: [u8; 4]) -> Self {
        Self { bytes }
    }

    /// Returns the underlying bytes of the selector.
    pub const fn to_bytes(self) -> [u8; 4] {
        self.bytes
    }
}
