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

    /// Resizes the static buffer to the given length.
    ///
    /// # Panics
    ///
    /// Panics for lengths greater than its capacity.
    pub fn resize(&mut self, new_len: usize) {
        if new_len > Self::CAPACITY {
            panic!("static buffer overflowed")
        }
        self.len = new_len;
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
