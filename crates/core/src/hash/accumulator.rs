// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

#[cfg(feature = "std")]
use ::std::io::{
    Result as IoResult,
    Write,
};
use ink_prelude::vec::Vec;

/// Hash builder that accumulates a buffer on the contract side.
pub trait Accumulator {
    /// Resets the buffer which cleans all state from it.
    ///
    /// # Note
    ///
    /// Useful when using `Vec` or similar as accumulator.
    fn reset(&mut self);
    /// Writes the given bytes into the buffer.
    fn write(&mut self, bytes: &[u8]);
    /// Returns a shared reference to the slice of the current state of the buffer.
    fn as_slice(&self) -> &[u8];
}

impl Accumulator for Vec<u8> {
    fn reset(&mut self) {
        self.clear()
    }

    fn write(&mut self, bytes: &[u8]) {
        // This could theoretically be speed-up by using `unsafe` `set_len`
        // and `[u8]` `copy_from_slice` methods.
        self.extend_from_slice(bytes)
    }

    fn as_slice(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'a, T> Accumulator for &'a mut T
where
    T: Accumulator,
{
    fn reset(&mut self) {
        <T as Accumulator>::reset(self)
    }

    fn write(&mut self, bytes: &[u8]) {
        <T as Accumulator>::write(self, bytes)
    }

    fn as_slice(&self) -> &[u8] {
        <T as Accumulator>::as_slice(self)
    }
}

/// Wraps a bytes buffer and turns it into an accumulator.
///
/// # Panics
///
/// Upon hash calculation if the underlying buffer length does not suffice the
/// needs of the accumulated hash buffer.
pub struct Wrap<'a> {
    /// The underlying wrapped buffer.
    buffer: &'a mut [u8],
    /// The current length of the filled area.
    len: usize,
}

impl Wrap<'_> {
    /// Returns the capacity of the underlying buffer.
    fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the length of the underlying buffer.
    fn len(&self) -> usize {
        self.len
    }
}

#[cfg(feature = "std")]
impl<'a> Write for Wrap<'a> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        <Self as Accumulator>::write(self, buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

impl<'a> From<&'a mut [u8]> for Wrap<'a> {
    fn from(buffer: &'a mut [u8]) -> Self {
        Self { buffer, len: 0 }
    }
}

impl<'a> Accumulator for Wrap<'a> {
    fn reset(&mut self) {
        self.len = 0;
    }

    fn write(&mut self, bytes: &[u8]) {
        debug_assert!(self.len() + bytes.len() <= self.capacity());
        let len = self.len;
        let bytes_len = bytes.len();
        self.buffer[len..(len + bytes_len)].copy_from_slice(bytes);
        self.len += bytes_len;
    }

    fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.len]
    }
}

#[cfg(not(feature = "std"))]
impl<'a> scale::Output for Wrap<'a> {
    fn write(&mut self, bytes: &[u8]) {
        <Self as Accumulator>::write(self, bytes)
    }

    fn push_byte(&mut self, byte: u8) {
        debug_assert!(self.len() < self.capacity());
        self.buffer[self.len] = byte;
        self.len += 1;
    }
}
