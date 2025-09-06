// Copyright (C) Use Ink (UK) Ltd.
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

/// A static buffer of variable capacity.
pub struct StaticBuffer {
    /// A static buffer of variable capacity.
    buffer: [u8; Self::CAPACITY],
}

impl StaticBuffer {
    /// The capacity of the static buffer.
    /// Usually set to 16 kB.
    /// Can be modified by setting `INK_STATIC_BUFFER_SIZE` environmental variable.
    const CAPACITY: usize = crate::BUFFER_SIZE;

    /// Creates a new static buffer.
    pub const fn new() -> Self {
        Self {
            buffer: [0; Self::CAPACITY],
        }
    }
}

impl core::ops::Index<core::ops::RangeFull> for StaticBuffer {
    type Output = [u8];

    #[inline(always)]
    fn index(&self, index: core::ops::RangeFull) -> &Self::Output {
        core::ops::Index::index(&self.buffer[..], index)
    }
}

impl core::ops::IndexMut<core::ops::RangeFull> for StaticBuffer {
    #[inline(always)]
    fn index_mut(&mut self, index: core::ops::RangeFull) -> &mut Self::Output {
        core::ops::IndexMut::index_mut(&mut self.buffer[..], index)
    }
}

/// Utility to allow for non-heap allocating encoding into a static buffer.
///
/// Required by `ScopedBuffer` internals.
pub struct EncodeScope<'a> {
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

impl scale::Output for EncodeScope<'_> {
    fn write(&mut self, bytes: &[u8]) {
        debug_assert!(
            self.len().checked_add(bytes.len()).unwrap() <= self.capacity(),
            "encode scope buffer overflowed. capacity is {} but last write index is {}",
            self.capacity(),
            self.len().checked_add(bytes.len()).unwrap(),
        );
        let start = self.len;
        let len_bytes = bytes.len();
        self.buffer[start..(start.checked_add(len_bytes)).unwrap()]
            .copy_from_slice(bytes);
        self.len = self.len.checked_add(len_bytes).unwrap();
    }

    fn push_byte(&mut self, byte: u8) {
        debug_assert_ne!(
            self.len(),
            self.capacity(),
            "encode scope buffer overflowed. capacity is {} and buffer is already full",
            self.capacity(),
        );
        self.buffer[self.len] = byte;
        self.len = self.len.checked_add(1).unwrap();
    }
}

/// Scoped access to an underlying bytes buffer.
///
/// # Note
///
/// This is used to efficiently chunk up ink!'s internal static 16 kB buffer
/// into smaller sub buffers for processing different parts of computations.
#[derive(Debug)]
pub struct ScopedBuffer<'a> {
    offset: usize,
    buffer: &'a mut [u8],
}

impl<'a> From<&'a mut [u8]> for ScopedBuffer<'a> {
    fn from(buffer: &'a mut [u8]) -> Self {
        Self { offset: 0, buffer }
    }
}

impl<'a> ScopedBuffer<'a> {
    /// Splits the scoped buffer into yet another piece to operate on it temporarily.
    ///
    /// The split buffer will have an offset of 0 but be offset by `self`'s offset.
    pub fn split(&mut self) -> ScopedBuffer<'_> {
        ScopedBuffer {
            offset: 0,
            buffer: &mut self.buffer[self.offset..],
        }
    }

    /// Returns the first `len` bytes of the buffer as mutable slice.
    pub fn take(&mut self, len: usize) -> &'a mut [u8] {
        debug_assert_eq!(self.offset, 0);
        debug_assert!(len <= self.buffer.len());
        let len_before = self.buffer.len();
        let buffer = core::mem::take(&mut self.buffer);
        let (lhs, rhs) = buffer.split_at_mut(len);
        self.buffer = rhs;
        debug_assert_eq!(lhs.len(), len);
        let len_after = self.buffer.len();
        debug_assert_eq!(len_before.checked_sub(len_after).unwrap(), len);
        lhs
    }

    /// Returns the first [`scale::MaxEncodedLen::max_encoded_len`] bytes of the buffer as
    /// a mutable slice.
    #[inline(always)]
    pub fn take_max_encoded_len<T>(&mut self) -> &'a mut [u8]
    where
        T: scale::MaxEncodedLen,
    {
        let len = T::max_encoded_len();
        self.take(len)
    }

    /// Encode the given value into the scoped buffer and return the sub slice
    /// containing all the encoded bytes.
    #[inline(always)]
    pub fn take_encoded<T>(&mut self, value: &T) -> &'a mut [u8]
    where
        T: scale::Encode,
    {
        debug_assert_eq!(self.offset, 0);
        let buffer = core::mem::take(&mut self.buffer);
        let mut encode_scope = EncodeScope::from(buffer);
        scale::Encode::encode_to(value, &mut encode_scope);
        let encode_len = encode_scope.len();
        let _ = core::mem::replace(&mut self.buffer, encode_scope.into_buffer());
        self.take(encode_len)
    }

    /// Encode the given value into the scoped buffer using the given encoder function
    /// and return the sub slice containing all the encoded bytes.
    pub fn take_encoded_with<F>(&mut self, encoder: F) -> &'a mut [u8]
    where
        F: FnOnce(&mut [u8]) -> usize,
    {
        let encode_len = encoder(self.buffer);
        self.take(encode_len)
    }

    /// Encode the given storable value into the scoped buffer and return the sub slice
    /// containing all the encoded bytes.
    #[inline(always)]
    pub fn take_storable_encoded<T>(&mut self, value: &T) -> &'a mut [u8]
    where
        T: ink_storage_traits::Storable,
    {
        debug_assert_eq!(self.offset, 0);
        let buffer = core::mem::take(&mut self.buffer);
        let mut encode_scope = EncodeScope::from(buffer);
        ink_storage_traits::Storable::encode(value, &mut encode_scope);
        let encode_len = encode_scope.len();
        let _ = core::mem::replace(&mut self.buffer, encode_scope.into_buffer());
        self.take(encode_len)
    }

    /// Appends the given bytes to the scoped buffer.
    ///
    /// Does not return the buffer immediately so that other values can be appended
    /// afterward. The [`take_appended`] method shall be used to return the buffer
    /// that includes all appended encodings as a single buffer.
    #[inline(always)]
    pub fn append_bytes(&mut self, bytes: &[u8]) {
        let offset = self.offset;
        let len = bytes.len();
        let end_offset = offset.checked_add(len).unwrap();
        self.buffer[offset..end_offset].copy_from_slice(bytes);
        self.offset = end_offset;
    }

    /// Returns the buffer containing all encodings appended via [`append_encoded`]
    /// in a single byte buffer.
    pub fn take_appended(&mut self) -> &'a mut [u8] {
        debug_assert_ne!(self.offset, 0);
        let offset = self.offset;
        self.offset = 0;
        self.take(offset)
    }

    /// Returns all remaining bytes of the buffer as a mutable slice.
    pub fn take_rest(self) -> &'a mut [u8] {
        debug_assert_eq!(self.offset, 0);
        debug_assert!(!self.buffer.is_empty());
        self.buffer
    }

    /// Returns the size of all remaining bytes in the buffer.
    ///
    /// # Developer Note
    ///
    /// The function requires `&mut self` because `self.buffer` is
    /// already a mutable reference in the struct.
    ///
    /// _The function does not actually mutate state._
    pub fn remaining_buffer(&mut self) -> usize {
        self.buffer.len()
    }
}
