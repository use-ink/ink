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

/// A static buffer with 16kB of capacity.
pub struct StaticBuffer {
    /// The static buffer with a total capacity of 16kB.
    buffer: [u8; Self::CAPACITY],
    /// The number of elements currently in use by the buffer
    /// counting from the start.
    len: usize,
}

impl StaticBuffer {
    /// The capacity of the static buffer.
    const CAPACITY: usize = 1 << 14; // 16kB

    /// Creates a new static buffer.
    pub const fn new() -> Self {
        Self {
            buffer: [0; Self::CAPACITY],
            len: 0,
        }
    }

    /// Returns the current length of the static buffer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Resets the length of the buffer to 0.
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl scale::Output for StaticBuffer {
    fn write(&mut self, bytes: &[u8]) {
        if self.len + bytes.len() > Self::CAPACITY {
            panic!("static buffer overflowed")
        }
        let start = self.len;
        let len_bytes = bytes.len();
        self.buffer[start..(start + len_bytes)].copy_from_slice(bytes);
        self.len += len_bytes;
    }

    fn push_byte(&mut self, byte: u8) {
        if self.len == Self::CAPACITY {
            panic!("static buffer overflowed")
        }
        self.buffer[self.len] = byte;
        self.len += 1;
    }
}

impl<I: core::slice::SliceIndex<[u8]>> core::ops::Index<I> for StaticBuffer {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        core::ops::Index::index(&self.buffer[..self.len], index)
    }
}

impl<I: core::slice::SliceIndex<[u8]>> core::ops::IndexMut<I> for StaticBuffer {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        core::ops::IndexMut::index_mut(&mut self.buffer[..self.len], index)
    }
}

struct EncodeScope<'a> {
    buffer: &'a mut [u8],
    len: usize,
}

impl<'a> From<&'a mut [u8]> for EncodeScope<'a> {
    fn from(buffer: &'a mut [u8]) -> Self {
        Self { buffer, len: 0 }
    }
}

impl<'a> EncodeScope<'a> {
    /// Returns the capacity of the encoded scope.
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the length of the encoded scope.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the internal mutable byte slice.
    pub fn into_buffer(self) -> &'a mut [u8] {
        self.buffer
    }
}

impl<'a> scale::Output for EncodeScope<'a> {
    fn write(&mut self, bytes: &[u8]) {
        assert!(self.len() + bytes.len() <= self.capacity());
        let start = self.len;
        let len_bytes = bytes.len();
        self.buffer[start..(start + len_bytes)].copy_from_slice(bytes);
        self.len += len_bytes;
    }

    fn push_byte(&mut self, byte: u8) {
        assert_ne!(self.len(), self.capacity());
        self.buffer[self.len] = byte;
        self.len += 1;
    }
}

/// Scoped access to an underlying bytes buffer.
///
/// # Note
///
/// This is used to efficiently chunk up ink!'s internal static 16kB buffer
/// into smaller sub buffers for processing different parts of computations.
#[derive(Debug)]
pub struct ScopedBuffer<'a> {
    buffer: &'a mut [u8],
}

impl<'a> From<&'a mut [u8]> for ScopedBuffer<'a> {
    fn from(buffer: &'a mut [u8]) -> Self {
        Self { buffer }
    }
}

impl<'a> ScopedBuffer<'a> {
    /// Returns the first `len` bytes of the buffer as mutable slice.
    pub fn take(&mut self, len: usize) -> &'a mut [u8] {
        assert!(len <= self.buffer.len());
        let len_before = self.buffer.len();
        let buffer = core::mem::take(&mut self.buffer);
        let (lhs, rhs) = buffer.split_at_mut(len);
        self.buffer = rhs;
        debug_assert_eq!(lhs.len(), len);
        let len_after = self.buffer.len();
        debug_assert_eq!(len_before - len_after, len);
        lhs
    }

    /// Encode the given value into the scoped buffer and return the sub slice
    /// containing all the encoded bytes.
    pub fn take_encoded<T>(&mut self, value: &T) -> &'a mut [u8]
    where
        T: scale::Encode,
    {
        let buffer = core::mem::take(&mut self.buffer);
        let mut encode_scope = EncodeScope::from(buffer);
        scale::Encode::encode_to(&value, &mut encode_scope);
        let encode_len = encode_scope.len();
        let _ = core::mem::replace(&mut self.buffer, encode_scope.into_buffer());
        self.take(encode_len)
    }

    /// Returns all of the remaining bytes of the buffer as mutable slice.
    pub fn take_rest(self) -> &'a mut [u8] {
        assert!(!self.buffer.is_empty());
        self.buffer
    }
}
