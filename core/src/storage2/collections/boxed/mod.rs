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

mod impls;
mod storage;

use crate::storage2::{
    alloc::{
        alloc,
        DynamicAllocation,
    },
    lazy::Lazy,
    traits::SpreadLayout,
};
use ink_primitives::Key;

/// An indirection to some dynamically allocated storage entity.
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
    /// if this did not happed before.
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
    /// if this did not happed before.
    ///
    /// # Panics
    ///
    /// If loading from contract storage failed.
    #[must_use]
    pub fn get_mut(boxed: &mut Self) -> &mut T {
        &mut boxed.value
    }
}
