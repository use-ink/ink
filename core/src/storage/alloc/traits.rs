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

use ink_primitives::Key;

/// Types implementing this trait can allocate storage.
///
/// # Note
///
/// Since the current Wasm implementation is 32-bit we are
/// fine to only support allocation sizes of max 32-bit in
/// contract storage. However, for static allocator like
/// `BumpAllocator` that is meant to allocate also other
/// allocators we might need relaxed allocation sizes.
pub trait Allocate {
    /// Allocates a storage area.
    ///
    /// The returned key denotes a storage region that fits for at
    /// least the given number of cells.
    fn alloc(&mut self, size: u64) -> Key;
}

/// Types implementing this trait are storage allocators.
pub trait Allocator: Allocate {
    /// Deallocates a storage area.
    ///
    /// The given storage region must have been allocated by this
    /// allocator before.
    fn dealloc(&mut self, key: Key);
}

/// Types implementing this trait can be allocated on the storage by storage allocators.
pub trait AllocateUsing
where
    Self: Sized,
{
    /// Allocates an uninitialized instance of `Self` using
    /// the given storage allocator.
    ///
    /// # Safety
    ///
    /// Unsafe because the storage contents of the resulting instance
    /// are uninitialized. Operating on uninitialized storage may result
    /// in panics or even in undefined behaviour.
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate;
}

/// Types implementing this trait require initialization of their storage contents
/// after allocation before they can be used.
///
/// # Example
///
/// For example types like [`Value`](struct.Value.html) have uninitialized
/// associated storage. Accessing a newly allocated instance of [`Value`](struct.Value.html)
/// would result in a panic or even undefined behaviour.
/// To circumvent this it is required to initialize its associated contract storage
/// via [`initialize`](trait.Initialize.html#method.initialize).
pub trait Initialize
where
    Self: Sized,
{
    /// Arguments used for deployment.
    ///
    /// # Note
    ///
    /// - This will probably most often be `()`.
    /// - For multiple arguments use tuples.
    type Args: Sized;

    /// The default value for default initialization purposes.
    ///
    /// Returns `None` by default which means that the type does not
    /// support default initialization.
    ///
    /// # Note
    ///
    /// Should be manually implemented only by those storage types
    /// that have a meaningful default initialization.
    ///
    /// # Example
    ///
    /// The `storage::Vec` is an example for which it makes sense to
    /// implement this to create empty `storage::Vec` by default that
    /// have their length field initialized to be `0`.
    #[inline(always)]
    fn default_value() -> Option<Self::Args> {
        None
    }

    /// Tries to default initialize `self`.
    ///
    /// # Note
    ///
    /// - Does nothing if `self` does not support default initialization.
    /// - This should never be manually implemented by any implementer.
    #[inline]
    fn try_default_initialize(&mut self) {
        if let Some(value) = Self::default_value() {
            self.initialize(value)
        }
    }

    /// Initializes storage of `self` so that it can be safely accessed.
    fn initialize(&mut self, args: Self::Args);

    /// Initializes storage of `self` so that it can be safely accessed.
    ///
    /// # Note
    ///
    /// Implementers should implement `initialize` instead of this.
    #[inline]
    fn initialize_into(self, args: Self::Args) -> Self {
        let mut this = self;
        this.initialize(args);
        this
    }
}
