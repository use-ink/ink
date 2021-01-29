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

use crate::call::Selector;
use ink_prelude::{
    vec,
    vec::Vec,
};

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

    fn encode_to<T: scale::Output + ?Sized>(&self, dest: &mut T) {
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
