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
use ink_prelude::{
    vec,
    vec::Vec,
};

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

/// The raw ABI respecting input data to a call.
///
/// # Note
///
/// The first four bytes are the function selector and the rest are SCALE encoded inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallData {
    /// Already encoded function selector and inputs.
    ///
    /// # Note
    ///
    /// Has the invariant of always holding at least 4 bytes (the selector).
    bytes: Vec<u8>,
}

impl CallData {
    /// Creates new call ABI data for the given selector.
    pub fn new(selector: Selector) -> Self {
        let bytes = selector.to_bytes();
        Self {
            bytes: vec![bytes[0], bytes[1], bytes[2], bytes[3]],
        }
    }

    /// Pushes the given argument onto the call ABI data in encoded form.
    pub fn push_arg<A>(&mut self, arg: &A)
    where
        A: scale::Encode,
    {
        arg.encode_to(&mut self.bytes)
    }

    /// Returns the selector of `self`.
    pub fn selector(&self) -> Selector {
        debug_assert!(self.bytes.len() >= 4);
        let bytes = [self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]];
        bytes.into()
    }

    /// Returns the underlying bytes of the encoded input parameters.
    pub fn params(&self) -> &[u8] {
        debug_assert!(self.bytes.len() >= 4);
        &self.bytes[4..]
    }

    /// Returns the underlying byte representation.
    pub fn to_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl scale::Encode for CallData {
    fn size_hint(&self) -> usize {
        self.bytes.len()
    }

    fn encode_to<T: scale::Output>(&self, dest: &mut T) {
        dest.write(self.bytes.as_slice());
    }
}

impl scale::Decode for CallData {
    fn decode<I: scale::Input>(
        input: &mut I,
    ) -> core::result::Result<Self, scale::Error> {
        let remaining_len = input.remaining_len().unwrap_or(None).unwrap_or(0);
        let mut bytes = Vec::with_capacity(remaining_len);
        while let Ok(byte) = input.read_byte() {
            bytes.push(byte);
        }
        if bytes.len() < 4 {
            return Err(scale::Error::from(
                "require at least 4 bytes for input data",
            ))
        }
        Ok(Self { bytes })
    }
}
