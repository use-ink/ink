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

use ink_primitives::Key;
use core::cell::UnsafeCell;

/// A lazy storage entity.
///
/// This loads its value from storage upon first use.
///
/// # Note
///
/// Use this if the storage field doesn't need to be loaded in some or most cases.
pub struct Lazy<T> {
    kind: UnsafeCell<LazyKind<T>>,
}

impl<T> Lazy<T> {
    /// Creates an eagerly populated lazy storage value.
    pub fn eager(value: T) -> Self {
        Self {
            kind: UnsafeCell::new(LazyKind::Occupied(OccupiedLazy::new(value))),
        }
    }

    /// Creates a true lazy storage value for the given key.
    pub fn lazy(key: Key) -> Self {
        Self {
            kind: UnsafeCell::new(LazyKind::Vacant(VacantLazy::new(key))),
        }
    }

    /// Returns a shared reference to the inner lazy kind.
    fn kind(&self) -> &LazyKind<T> {
        unsafe { &*self.kind.get() }
    }

    /// Returns an exclusive reference to the inner lazy kind.
    fn kind_mut(&mut self) -> &mut LazyKind<T> {
        unsafe { &mut *self.kind.get() }
    }

    /// Performs the given closure on the mutable lazy kind.
    ///
    /// # Note
    ///
    /// Actions on the mutable lazy kind are performed within the closure
    /// to not leak exclusive references to it to the outside. This is important
    /// since the `for_kind` method itself operates only on `&self`.
    fn for_kind<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut LazyKind<T>) -> R,
    {
        f(unsafe { &mut *self.kind.get() })
    }
}

impl<T> Lazy<T>
where
    T: scale::Decode,
{
    /// Loads the value lazily from contract storage.
    ///
    /// Does nothing if value has already been loaded.
    fn load_value_lazily(&self) {
        self.for_kind(|kind| {
            if let LazyKind::Vacant(vacant) = kind {
                let value = crate::env::get_contract_storage::<T>(vacant.key)
                    .expect("couldn't find contract storage entry")
                    .expect("couldn't properly decode contract storage entry");
                *kind = LazyKind::Occupied(OccupiedLazy::new(value));
            }
        });
    }

    /// Returns a shared reference to the lazily loaded value.
    ///
    /// # Note
    ///
    /// This loads the value from the contract storage if this did not happed before.
    ///
    /// # Panics
    ///
    /// If loading from contract storage failed.
    pub fn get(&self) -> &T {
        self.load_value_lazily();
        match self.kind() {
            LazyKind::Vacant(_) => panic!("expect occupied lazy here"),
            LazyKind::Occupied(occupied) => &occupied.value,
        }
    }

    /// Returns an exclusive reference to the lazily loaded value.
    ///
    /// # Note
    ///
    /// This loads the value from the contract storage if this did not happed before.
    ///
    /// # Panics
    ///
    /// If loading from contract storage failed.
    pub fn get_mut(&mut self) -> &mut T {
        self.load_value_lazily();
        match self.kind_mut() {
            LazyKind::Vacant(_) => panic!("expect occupied lazy here"),
            LazyKind::Occupied(occupied) => &mut occupied.value,
        }
    }
}

impl<T> core::ops::Deref for Lazy<T>
where
    T: scale::Decode,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> core::ops::DerefMut for Lazy<T>
where
    T: scale::Decode,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

/// The lazy storage entity can be in either of two states.
///
/// 1. It either is vacant and thus a real lazy storage entity that
///    waits until it is used for the first time in order to load its value
///    from the contract storage.
/// 2. It is actually an already occupied eager lazy.
pub enum LazyKind<T> {
    /// A true lazy storage entity that loads its contract storage value upon first use.
    Vacant(VacantLazy),
    /// An already loaded eager lazy storage entity.
    Occupied(OccupiedLazy<T>),
}

/// The lazy storage entity is in a lazy state.
pub struct VacantLazy {
    /// The key to load the value from contract storage upon first use.
    pub key: Key,
}

impl VacantLazy {
    /// Creates a new truly lazy storage entity for the given key.
    pub fn new(key: Key) -> Self {
        Self { key }
    }
}

/// An already loaded or otherwise occupied eager lazy storage entity.
pub struct OccupiedLazy<T> {
    /// The loaded value.
    pub value: T,
}

impl<T> OccupiedLazy<T> {
    /// Creates a new eager lazy storage entity with the given value.
    pub fn new(value: T) -> Self {
        Self { value }
    }
}
