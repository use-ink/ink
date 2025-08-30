// Copyright (C) ink! contributors.
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

//! Abstractions for ABI representation and encoding/decoding.

use ink_prelude::vec::Vec;

use crate::sol::{
    SolDecode,
    SolEncode,
};

/// ABI spec for encoding/decoding contract calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Abi {
    /// ink! ABI spec (uses Parity's SCALE codec for input/output encode/decode).
    Ink,
    /// Solidity ABI encoding.
    Sol,
}

/// Marker type for ink! ABI and SCALE encoding.
///
/// Used with [`AbiEncodeWith`], [`AbiDecodeWith`] and `DecodeMessageResult`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Ink;

/// Marker type for Solidity ABI.
///
/// Used with [`AbiEncodeWith`], [`AbiDecodeWith`] and `DecodeMessageResult`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Sol;

/// Trait for ABI-specific encoding with support for both slice and vector buffers.
pub trait AbiEncodeWith<Abi> {
    /// Encodes the data into a new vector.
    fn encode_with(&self) -> Vec<u8>;

    /// Encodes the data into a fixed-size buffer, returning the number of bytes written.
    fn encode_to_slice(&self, buffer: &mut [u8]) -> usize;

    /// Encodes the data into a dynamically resizing vector.
    fn encode_to_vec(&self, buffer: &mut Vec<u8>);

    /// Encodes the value as a topic (i.e. an indexed event parameter).
    ///
    /// # Note
    ///
    /// The provided hashing function depends on the ABI i.e.
    /// - BLAKE2b for ink! ABI
    /// - Keccak-256 for Solidity ABI
    fn encode_topic<H>(&self, hasher: H) -> [u8; 32]
    where
        H: Fn(&[u8], &mut [u8; 32]);
}

/// Trait for ABI-specific decoding.
pub trait AbiDecodeWith<Abi>: Sized {
    /// The error type that can occur during decoding.
    type Error: core::fmt::Debug;
    /// Decodes the data from a buffer using the provided ABI.
    fn decode_with(buffer: &[u8]) -> Result<Self, Self::Error>;
}

impl<T: scale::Encode> AbiEncodeWith<Ink> for T {
    fn encode_with(&self) -> Vec<u8> {
        scale::Encode::encode(self)
    }

    fn encode_to_slice(&self, buffer: &mut [u8]) -> usize {
        let encoded = scale::Encode::encode(self);
        let len = encoded.len();
        debug_assert!(
            len <= buffer.len(),
            "encode scope buffer overflowed, encoded len is {} but buffer len is {}",
            len,
            buffer.len()
        );
        buffer[..len].copy_from_slice(&encoded);
        len
    }

    fn encode_to_vec(&self, buffer: &mut Vec<u8>) {
        scale::Encode::encode_to(self, buffer);
    }

    fn encode_topic<H>(&self, hasher: H) -> [u8; 32]
    where
        H: Fn(&[u8], &mut [u8; 32]),
    {
        let encoded = scale::Encode::encode(self);
        let len = encoded.len();
        let mut output = [0u8; 32];
        if encoded.len() <= 32 {
            output.as_mut()[0..len].copy_from_slice(&encoded);
        } else {
            hasher(&encoded, &mut output);
        }
        output
    }
}

impl<T: scale::Decode> AbiDecodeWith<Ink> for T {
    type Error = scale::Error;
    fn decode_with(buffer: &[u8]) -> Result<Self, Self::Error> {
        scale::Decode::decode(&mut &buffer[..])
    }
}

impl<T> AbiEncodeWith<Sol> for T
where
    T: for<'a> SolEncode<'a>,
{
    fn encode_with(&self) -> Vec<u8> {
        SolEncode::encode(self)
    }

    fn encode_to_slice(&self, buffer: &mut [u8]) -> usize {
        let encoded = SolEncode::encode(self);
        let len = encoded.len();
        debug_assert!(
            len <= buffer.len(),
            "encode scope buffer overflowed, encoded len is {} but buffer len is {}",
            len,
            buffer.len()
        );
        buffer[..len].copy_from_slice(&encoded);
        len
    }

    fn encode_to_vec(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&T::encode(self));
    }

    fn encode_topic<H>(&self, hasher: H) -> [u8; 32]
    where
        H: Fn(&[u8], &mut [u8; 32]),
    {
        SolEncode::encode_topic(self, hasher)
    }
}

impl<T: SolDecode> AbiDecodeWith<Sol> for T {
    type Error = crate::sol::Error;
    fn decode_with(buffer: &[u8]) -> Result<Self, Self::Error> {
        T::decode(buffer)
    }
}
