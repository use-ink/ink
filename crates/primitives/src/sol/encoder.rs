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

/// A Solidity ABI encoder.
//
// # Design Notes
//
// In contrast to `alloy_sol_types::abi::Encoder`, this implementation is non-allocating.
//
// Note though, that this non-allocating claim is about the encoder itself, not the
// representations of the types it encodes (i.e. types with allocating representations
// like `Vec<T>` inherently allocate).
pub struct Encoder<'enc> {
    /// The head buffer.
    head: &'enc mut [u8],
    /// The (segmented) tail buffer.
    tail: Option<&'enc mut [u8]>,
    /// The head offset.
    head_offset: usize,
    /// The tail offset.
    tail_offset: usize,
}

impl<'enc> Encoder<'enc> {
    /// Creates an encoder from a mutable byte slice.
    pub fn new(buffer: &'enc mut [u8]) -> Self {
        Self {
            head: buffer,
            tail: None,
            head_offset: 0,
            tail_offset: 0,
        }
    }

    /// Appends a word.
    pub fn append_word(&mut self, word: [u8; 32]) {
        debug_assert_eq!(self.head_offset % 32, 0);
        let next_offset = self.head_offset.checked_add(32).unwrap();
        self.head[self.head_offset..next_offset].copy_from_slice(word.as_slice());
        debug_assert_eq!(next_offset % 32, 0);
        self.head_offset = next_offset;
    }

    /// Appends bytes.
    pub fn append_bytes(&mut self, bytes: &[u8]) {
        debug_assert_eq!(self.head_offset % 32, 0);
        if bytes.is_empty() {
            return;
        }
        let end_offset = self.head_offset.checked_add(bytes.len()).unwrap();
        self.head[self.head_offset..end_offset].copy_from_slice(bytes);
        let next_offset = match end_offset % 32 {
            0 => end_offset,
            r => {
                let pad_len = 32 - r;
                let next_offset = end_offset.checked_add(pad_len).unwrap();
                self.head[end_offset..next_offset].fill(0u8);
                next_offset
            }
        };
        debug_assert_eq!(next_offset % 32, 0);
        self.head_offset = next_offset;
    }

    /// Appends offset.
    ///
    /// # Note
    ///
    /// This method should be called after segmenting the buffer using [`Self::segment`].
    pub fn append_offset(&mut self) {
        debug_assert!(self.tail.is_some());
        debug_assert_eq!(self.tail_offset % 32, 0);
        // The "overall" offset for dynamic data combines the head length and current
        // offset in the tail buffer.
        let offset = self.head.len().checked_add(self.tail_offset).unwrap();
        self.append_as_be_bytes(offset);
    }

    /// Appends length of a sequence.
    pub fn append_length(&mut self, len: usize) {
        self.append_as_be_bytes(len);
    }

    /// Segments the buffer into a head and tail, with the head taking the next `n` words.
    pub fn segment(&mut self, n_words: usize) -> Encoder<'_> {
        debug_assert_eq!(self.head_offset % 32, 0);
        let (_, buffer) = self.head.split_at_mut(self.head_offset);
        let (head, tail) = buffer.split_at_mut(n_words.checked_mul(32).unwrap());
        Encoder {
            head,
            tail: Some(tail),
            head_offset: 0,
            tail_offset: 0,
        }
    }

    /// Takes `n` words from the tail buffer.
    ///
    /// # Note
    ///
    /// This method must be called after segmenting the buffer using [`Self::segment`].
    ///
    /// # Panics
    ///
    /// Panics if the buffer isn't segmented.
    pub fn take_tail(&mut self, n_words: usize) -> Encoder<'_> {
        let tail = core::mem::take(&mut self.tail)
            .expect("Expected a segmented buffer, call `Self::segment` first");
        let len = n_words.checked_mul(32).unwrap();
        let (target, rest) = tail.split_at_mut(len);
        self.tail = Some(rest);
        self.tail_offset = self.tail_offset.checked_add(len).unwrap();
        debug_assert_eq!(self.tail_offset % 32, 0);
        Encoder::new(target)
    }

    /// Fills the next `n` words with the given value.
    pub fn fill(&mut self, value: u8, n_words: usize) {
        debug_assert_eq!(self.head_offset % 32, 0);
        let end_offset = self
            .head_offset
            .checked_add(n_words.checked_mul(32).unwrap())
            .unwrap();
        self.head[self.head_offset..end_offset].fill(value);
        self.head_offset = end_offset;
    }

    /// Appends the big endian bytes for value (e.g. an offset or length).
    fn append_as_be_bytes(&mut self, len: usize) {
        debug_assert_eq!(self.head_offset % 32, 0);
        let bytes = len.to_be_bytes();
        // `usize` can't theoretically be any larger than 128 bits (16 bytes),
        // and practically it's never more than 64 bits (8 bytes).
        let end_offset = self.head_offset.checked_add(32).unwrap();
        let start_offset = end_offset.checked_sub(bytes.len()).unwrap();
        self.head[self.head_offset..start_offset].fill(0);
        self.head[start_offset..end_offset].copy_from_slice(bytes.as_slice());
        debug_assert_eq!(end_offset % 32, 0);
        self.head_offset = end_offset;
    }
}
