// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use crate::{
    env2::utils::{
        EnlargeTo,
        Reset,
    },
    memory::vec::Vec,
};
use core::cell::{Cell, RefCell};

//! This file defines the global buffer arena that is accessible globally
//! and acts as a cache for allocated heap memory to avoid heap memory
//! throttling resulting from heap usage for intermediate computations.
//!
//! Exactly this happens a lot in the boundaries between SRML contracts
//! and ink! since encoding and decoding of SCALE values has to be done
//! in such an intermediate buffer.
//!
//! Users and systems are advised to share a common set of allocated buffers
//! provided by the global buffer arena.

/// The maximum amount of used byte buffers at the same time.
///
/// Since the whole point behind this byte buffer arena is to cache
/// allocated heap memory the number of concurrent byte buffers that are
/// in use at the same time should be kept small.
const IN_USE_LIMIT: usize = 1000;

thread_local! {
    /// Global buffer arena that provides shared buffers to avoid
    /// constantly allocating and deallocating heap memory during
    /// contract execution.
    ///
    /// This is mainly useful to interact with the environment because
    /// it requires intermediate buffers for its encoding and decoding.
    ///
    /// Provides a single API `get_buffer` that can be used to get
    /// a freshly recycled buffer.
    pub static BUFFER_ARENA: BufferArena = BufferArena::new();
}

/// A byte buffer arena that acts as a cache for allocated heap memory.
pub struct BufferArena {
    /// The currently available byte buffers.
    free: RefCell<Vec<Buffer>>,
    /// Counts the buffers that are currently in use at the same time.
    in_use: Cell<usize>,
    /// The number of allocated buffers owned by the buffer arena.
    ///
    /// This is always the same as the maximum number of buffers in use
    /// at the same time until the point of evaluation.
    allocated: Cell<usize>,
}

impl BufferArena {
    /// Returns a new empty buffer arena.
    ///
    /// Since this acts as cache we only require one instance of this type
    /// that we use as `thread_local` global which is safe since
    /// Wasm smart contracts are guaranteed to run in a single thread.
    pub(in self) fn new() -> Self {
        Self {
            free: RefCell::new(Vec::new()),
            in_use: Cell::new(0),
            allocated: Cell::new(0),
        }
    }

    /// Returns a fresh buffer that can be used for intermediate computations.
    ///
    /// Buffers returned through this API implement all the necessary traits
    /// required to use them as environment buffer.
    ///
    /// - [`Reset`]: Allows resetting the buffer. This clears all the elements,
    ///            however, it retains the memory allocation.
    /// - [`EnlargeTo`]: Safely enlarges the buffer to the required minimum size
    ///                  if it isn't already large enough.
    /// - [`AsRef<[u8]>`]: Returns a shared view into the byte buffer.
    /// - [`AsMut<[u8]>`]: Returns an exclusive view into the byte buffer.
    pub fn get_buffer(&self) -> BufferRef {
        let in_use = self.in_use.update(|x| x + 1);
        self.allocated.update(|x| core::cmp::max(x, in_use));
        if in_use > IN_USE_LIMIT {
            panic!("too many concurrent byte buffers")
        }
        self.free
            .borrow_mut()
            .pop()
            .unwrap_or(Buffer::new())
            .into_ref()
    }

    /// Returns the buffer to the arena.
    ///
    /// This is only called from the `Drop` implementation of `BufferRef`
    /// to return the wrapped buffer back to the global buffer arena instance.
    pub(in self) fn return_buffer(&self, buffer: Buffer) {
        self.in_use.update(|x| x - 1);
        self.free.borrow_mut().push(buffer)
    }
}

/// A byte buffer.
///
/// This is a thin wrapper around a byte vector providing only
/// the minimal interface to be operable as environmental intermediate
/// buffer.
pub struct Buffer {
    /// The warpper internal raw byte buffer.
    buffer: Vec<u8>,
}

impl<'a> Buffer
where
    Self: 'a,
{
    /// Returns a new empty byte buffer.
    pub(in self) const fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Wraps `self` in a buffer reference.
    pub(in self) fn into_ref(self) -> BufferRef<'a> {
        BufferRef { buffer: self, lt: core::marker::PhantomData }
    }
}

impl Reset for Buffer {
    fn reset(&mut self) {
        Reset::reset(&mut self.buffer)
    }
}

impl EnlargeTo for Buffer {
    fn enlarge_to(&mut self, new_size: usize) {
        EnlargeTo::enlarge_to(&mut self.buffer, new_size)
    }
}

impl core::convert::AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        &self.buffer
    }
}

impl core::convert::AsMut<[u8]> for Buffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

/// A buffer reference providing lifetime constrained access
/// to the wrapper byte buffer.
///
/// Buffer references are created by the only global buffer arena
/// instance and make sure that their wrapper buffer is returned to
/// the arena upon `Drop`.
pub struct BufferRef<'a> {
    /// The wrapped byte buffer.
    buffer: Buffer,
    /// The emulated lifetime.
    lt: core::marker::PhantomData<fn () -> &'a ()>,
}

impl BufferRef<'_> {
    /// Takes the buffer out of the buffer reference
    /// leaving behind an empty byte buffer without
    /// associated heap allocation.
    ///
    /// Also resets the byte buffer.
    fn take_buffer(&mut self) -> Buffer {
        Reset::reset(&mut self.buffer);
        core::mem::replace(&mut self.buffer, Buffer::new())
    }
}

impl Reset for BufferRef<'_> {
    fn reset(&mut self) {
        Reset::reset(&mut self.buffer)
    }
}

impl EnlargeTo for BufferRef<'_> {
    fn enlarge_to(&mut self, new_size: usize) {
        EnlargeTo::enlarge_to(&mut self.buffer, new_size)
    }
}

impl core::convert::AsRef<[u8]> for BufferRef<'_> {
    fn as_ref(&self) -> &[u8] {
        core::convert::AsRef::<[u8]>::as_ref(&self.buffer)
    }
}

impl core::convert::AsMut<[u8]> for BufferRef<'_> {
    fn as_mut(&mut self) -> &mut [u8] {
        core::convert::AsMut::<[u8]>::as_mut(&mut self.buffer)
    }
}

impl Drop for BufferRef<'_> {
    fn drop(&mut self) {
        // Returns the byte buffer back to the global buffer arena.
        BUFFER_ARENA.with(|arena| arena.return_buffer(self.take_buffer()))
    }
}
