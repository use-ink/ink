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

use derive_more::From;

/// Seals to guard pushing arguments to already satisfied parameter builders.
pub mod seal {
    /// The call builder is sealed and won't accept further arguments.
    pub enum Sealed {}
    /// The call builder is unsealed and will accept further arguments.
    pub enum Unsealed {}
}

/// The function selector.
#[derive(Debug, Copy, Clone, PartialEq, Eq, From, scale::Decode, scale::Encode)]
pub struct Selector {
    /// The 4 underlying bytes.
    bytes: [u8; 4],
}

impl<'a> From<&'a [u8]> for Selector {
    /// Computes the selector from the given input bytes.
    ///
    /// # Note
    ///
    /// Normally this is invoked through `Selector::from_str`.
    fn from(input: &'a [u8]) -> Self {
        let keccak = ink_primitives::hash::keccak256(input);
        Self {
            bytes: [keccak[0], keccak[1], keccak[2], keccak[3]],
        }
    }
}

impl Selector {
    /// Returns the selector for the given name.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(name: &str) -> Self {
        From::from(name.as_bytes())
    }

    /// Creates a selector directly from 4 bytes.
    pub const fn new(bytes: [u8; 4]) -> Self {
        Self { bytes }
    }

    /// Returns the underlying bytes of the selector.
    pub const fn to_bytes(self) -> [u8; 4] {
        self.bytes
    }
}
