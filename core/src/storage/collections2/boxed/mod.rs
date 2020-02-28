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

mod traits;

use crate::storage::{
    ClearForward,
    Lazy,
    PullForward,
    StorageSize,
};
use ink_primitives::Key;

/// Allocates a new storage key for the given `T` dynamically on the storage.
fn allocate_dynamically<T>() -> Key
where
    T: StorageSize,
{
    // TODO: Actual implementation is still missing!
    Key([0x42; 32])
}

/// An indirection to some dynamically allocated storage entity.
pub struct Box<T>
where
    T: ClearForward,
{
    /// The storage area where the boxed storage entity is stored.
    key: Key,
    /// The cache for the boxed storage entity.
    value: Lazy<T>,
}

impl<T> Box<T>
where
    T: ClearForward + StorageSize,
{
    /// Creates a new boxed entity.
    pub fn new(value: T) -> Self {
        Self {
            key: allocate_dynamically::<T>(),
            value: Lazy::new(value),
        }
    }
}

impl<T> Box<T>
where
    T: ClearForward + StorageSize + PullForward,
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
    pub fn get(&self) -> &T {
        self.value.get()
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
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }
}
