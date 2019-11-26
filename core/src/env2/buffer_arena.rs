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

use crate::{
    env2::utils::{
        EnlargeTo,
        Reset,
    },
    memory::vec::Vec,
};
use cfg_if::cfg_if;
use core::cell::{
    Cell,
    RefCell,
};

/// The maximum amount of used byte buffers at the same time.
///
/// Since the whole point behind this byte buffer arena is to cache
/// allocated heap memory the number of concurrent byte buffers that are
/// in use at the same time should be kept small.
const IN_USE_LIMIT: usize = 1000;

cfg_if! {
    if #[cfg(feature = "std")] {
        thread_local! {
            /// Global buffer arena that provides shared recycled buffers to avoid
            /// constantly allocating and deallocating heap memory during contract execution.
            ///
            /// # Note
            ///
            /// This is mainly useful to interact with the environment because
            /// it requires intermediate buffers for its encoding and decoding.
            ///
            /// # API
            ///
            /// - Can be accessed through [`std::thread::LocalKey::with`].
            /// - Provides recycled buffers through the `get_buffer` call.
            pub static BUFFER_ARENA: BufferArena = BufferArena::new();
        }
    } else {
        /// Global buffer arena that provides shared recycled buffers to avoid
        /// constantly allocating and deallocating heap memory during contract execution.
        ///
        /// # Note
        ///
        /// This is mainly useful to interact with the environment because
        /// it requires intermediate buffers for its encoding and decoding.
        ///
        /// # API
        ///
        /// - Can be accessed through [`std::thread::LocalKey::with`].
        /// - Provides recycled buffers through the `get_buffer` call.
        pub static BUFFER_ARENA: GlobalBufferArena = GlobalBufferArena::new(BufferArena::new());

        /// Wrapper around `BufferArena` to provide similar interface
        /// as `std::thread::LocalKey` provided by `thread_local` does.
        ///
        /// Also acts as safety guard to prevent references to `BufferRef`
        /// escape the closure using the [`GlobalBufferArena::with`] API.
        pub struct GlobalBufferArena {
            /// The wrapped buffer arena.
            arena: BufferArena,
        }

        /// CRITICAL NOTE
        /// =============
        ///
        /// The wrapped `BufferArena` type itself is __NOT__ `Sync` since it is using
        /// `Cell` and `RefCell` internally instead of the thread-safe alternatives.
        /// However, since Wasm smart contracts are guaranteed to operate single
        /// threaded we can allow for this unsafe `Sync` implementation to allow
        /// for having the global static `BUFFER_ARENA` variable and as long as we
        /// are only operating single threaded this shouldn't be unsafe.
        unsafe impl Sync for GlobalBufferArena {}

        impl GlobalBufferArena {
            /// Creates a new `GlobalBufferArena` from the given `BufferArena`.
            pub const fn new(arena: BufferArena) -> Self {
                Self { arena }
            }

            /// Runs the given closure for the wrapped `BufferArena`.
            ///
            /// This way no references may escape the closure.
            pub fn with<F>(&self, f: F)
            where
                F: FnOnce(&BufferArena),
            {
                f(&self.arena)
            }
        }
    }
}

/// A byte buffer arena that acts as a cache for allocated heap memory.
pub struct BufferArena {
    /// The currently available byte buffers.
    free: RefCell<Vec<Buffer>>,
    /// Counts the buffers that are currently in use at the same time.
    ///
    /// # Note
    ///
    /// This value is purely used as diagnostic measures to provide
    /// smart contract writers with feedback if their implementation
    /// is abusing the buffer arena.
    /// We might want to turn these checks off for Wasm compilation.
    in_use: Cell<usize>,
}

impl BufferArena {
    /// Returns a new empty buffer arena.
    ///
    /// Since this acts as cache we only require one instance of this type
    /// that we use as `thread_local` global which is safe since
    /// Wasm smart contracts are guaranteed to run in a single thread.
    pub(self) const fn new() -> Self {
        Self {
            free: RefCell::new(Vec::new()),
            in_use: Cell::new(0),
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
    /// - [`core::convert::AsRef`]`<[u8]>`: Returns a shared view into the byte buffer.
    /// - [`core::convert::AsMut`]`<[u8]>`: Returns an exclusive view into the byte buffer.
    pub fn get_buffer(&self) -> BufferRef {
        self.in_use.set(self.in_use() + 1);
        if self.in_use() > IN_USE_LIMIT {
            panic!("too many concurrent byte buffers")
        }
        self.free
            .borrow_mut()
            .pop()
            .unwrap_or_else(Buffer::new)
            .into_ref()
    }

    /// Returns the buffer to the arena.
    ///
    /// This is only called from the `Drop` implementation of `BufferRef`
    /// to return the wrapped buffer back to the global buffer arena instance.
    pub(self) fn return_buffer(&self, buffer: Buffer) {
        self.in_use.set(self.in_use() - 1);
        self.free.borrow_mut().push(buffer)
    }

    /// Returns the number of buffers that are currently in use at the same time.
    pub fn in_use(&self) -> usize {
        self.in_use.get()
    }

    /// Returns the number of buffers that are not in use.
    pub fn free(&self) -> usize {
        self.free.borrow().len()
    }

    /// Returns the current number of cached buffers.
    pub fn allocated(&self) -> usize {
        self.in_use() + self.free()
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
    pub(self) const fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Wraps `self` in a buffer reference.
    pub(self) fn into_ref(self) -> BufferRef<'a> {
        BufferRef {
            buffer: self,
            lt: core::marker::PhantomData,
        }
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
    lt: core::marker::PhantomData<fn() -> &'a ()>,
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_arena {
        (
            $arena:ident,
            in_use: $expected_in_use:literal,
            free: $expected_free:literal,
            allocated: $expected_allocated:literal
        ) => {{
            assert_eq!(
                $arena.in_use(),
                $expected_in_use,
                "number of buffers in use doesn't match expected"
            );
            assert_eq!(
                $arena.free(),
                $expected_free,
                "number of free buffers doens't match expected"
            );
            assert_eq!(
                $arena.allocated(),
                $expected_allocated,
                "number of allocated buffers doesn't match expected"
            );
        }};
    }

    #[test]
    fn it_works() {
        BUFFER_ARENA.with(|arena| {
            assert_arena!(arena, in_use: 0, free: 0, allocated: 0);
            // Allocate a single buffer for a short time.
            {
                let _b = arena.get_buffer();
                assert_arena!(arena, in_use: 1, free: 0, allocated: 1);
            }
            // We should now have a single allocated buffer
            // but none in use.
            assert_arena!(arena, in_use: 0, free: 1, allocated: 1);
            // Allocate a single buffer again so that we see
            // it is being reused.
            {
                let _b = arena.get_buffer();
                assert_arena!(arena, in_use: 1, free: 0, allocated: 1);
            }
            assert_arena!(arena, in_use: 0, free: 1, allocated: 1);
            // Now we allocate 3 buffers in their own scope
            // and check the `in_use` and `allocated`.
            {
                let _b0 = arena.get_buffer();
                {
                    let _b1 = arena.get_buffer();
                    {
                        // At this point we should have 3 buffers
                        // allocated and in use.
                        let _b2 = arena.get_buffer();
                        assert_arena!(arena, in_use: 3, free: 0, allocated: 3);
                    }
                    assert_arena!(arena, in_use: 2, free: 1, allocated: 3);
                }
                assert_arena!(arena, in_use: 1, free: 2, allocated: 3);
            }
            // At this point we dropped all 3 buffers again
            // so none is in use but we still have 3 allocated
            // buffers.
            assert_arena!(arena, in_use: 0, free: 3, allocated: 3);
        });
    }
}
