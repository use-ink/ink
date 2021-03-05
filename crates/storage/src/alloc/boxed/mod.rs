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

mod impls;
mod storage;

#[cfg(test)]
mod tests;

use crate::{
    alloc::{
        alloc,
        DynamicAllocation,
    },
    lazy::Lazy,
    traits::SpreadLayout,
};
use ink_primitives::Key;

/// A dynamically allocated storage entity.
///
/// Users can use this in order to make certain `SpreadLayout` storage entities
/// used in contexts that require a `PackedLayout` storage entity by simply
/// packing the storage entity within a `storage::Box`.
///
/// Dynamic allocations caused by the creation of `storage::Box` instances do
/// have some limited overhead:
///
/// - The dynamic allocation itself has to be provided by some dynamic storage
///   allocator that needs to be invoked.
/// - Each dynamic storage allocation implies roughly 1.12 bits of overhead.
/// - Upon ever first dereferencing of a `storage::Box` instance a cryptographic
///   hash routine is run in order to compute the underlying storage key.
///
/// Use this abstraction with caution due to the aforementioned performance
/// implications.
#[derive(Debug)]
pub struct Box<T>
where
    T: SpreadLayout,
{
    /// The storage area where the boxed storage entity is stored.
    allocation: DynamicAllocation,
    /// The cache for the boxed storage entity.
    value: Lazy<T>,
}

impl<T> Box<T>
where
    T: SpreadLayout,
{
    /// Creates a new boxed entity.
    pub fn new(value: T) -> Self {
        Self {
            allocation: alloc(),
            value: Lazy::new(value),
        }
    }

    /// Creates a new boxed entity that has not yet loaded its value.
    fn lazy(allocation: DynamicAllocation) -> Self {
        Self {
            allocation,
            value: Lazy::lazy(allocation.key()),
        }
    }

    /// Returns the underlying storage key for the dynamic allocated entity.
    fn key(&self) -> Key {
        self.allocation.key()
    }
}

impl<T> Box<T>
where
    T: SpreadLayout,
{
    /// Returns a shared reference to the boxed value.
    ///
    /// # Note
    ///
    /// This loads the value from the pointed to contract storage
    /// if this did not happen before.
    ///
    /// # Panics
    ///
    /// If loading from contract storage failed.
    #[must_use]
    pub fn get(boxed: &Self) -> &T {
        &boxed.value
    }

    /// Returns an exclusive reference to the boxed value.
    ///
    /// # Note
    ///
    /// This loads the value from the pointed to contract storage
    /// if this did not happen before.
    ///
    /// # Panics
    ///
    /// If loading from contract storage failed.
    #[must_use]
    pub fn get_mut(boxed: &mut Self) -> &mut T {
        &mut boxed.value
    }
}
