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

use super::{
    KeyPtr,
    Pull,
    Push,
    StorageSize,
};
use core::cell::UnsafeCell;
use ink_primitives::Key;

/// A lazy storage entity.
///
/// This loads its value from storage upon first use.
///
/// # Note
///
/// Use this if the storage field doesn't need to be loaded in some or most cases.
#[derive(Debug)]
pub struct Lazy<T> {
    kind: UnsafeCell<LazyKind<T>>,
}

impl<T> StorageSize for Lazy<T>
where
    T: StorageSize,
{
    const SIZE: u64 = <T as StorageSize>::SIZE;
}

impl<T> Pull for Lazy<T>
where
    T: StorageSize + scale::Decode,
{
    fn pull(key_ptr: &mut KeyPtr) -> Self {
        Self::lazy(key_ptr.next_for::<T>())
    }
}

impl<T> Push for Lazy<T>
where
    T: Push,
{
    fn push(&self, key_ptr: &mut KeyPtr) {
        // We skip pushing to contract storage if we are still in unloaded form.
        if let LazyKind::Occupied(occupied) = self.kind() {
            occupied.value.push(key_ptr)
        }
    }
}

impl<T> Lazy<T> {
    /// Creates an eagerly populated lazy storage value.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self {
            kind: UnsafeCell::new(LazyKind::Occupied(OccupiedLazy::new(value))),
        }
    }

    /// Creates a true lazy storage value for the given key.
    #[must_use]
    pub fn lazy(key: Key) -> Self {
        Self {
            kind: UnsafeCell::new(LazyKind::Vacant(VacantLazy::new(key))),
        }
    }

    /// Returns a shared reference to the inner lazy kind.
    #[must_use]
    fn kind(&self) -> &LazyKind<T> {
        // SAFETY: We just return a shared reference while the method receiver
        //         is a shared reference (&self) itself. So we respect normal
        //         Rust rules.
        unsafe { &*self.kind.get() }
    }

    /// Returns an exclusive reference to the inner lazy kind.
    #[must_use]
    fn kind_mut(&mut self) -> &mut LazyKind<T> {
        // SAFETY: We just return an exclusive reference while the method receiver
        //         is an exclusive reference (&mut self) itself. So we respect normal
        //         Rust rules.
        unsafe { &mut *self.kind.get() }
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
        // SAFETY: We mutate the kind only if it is vacant.
        //         So if there is an actual value (Occupied) we leave the
        //         entire entity as it is not to invalidate references to it.
        let kind = unsafe { &mut *self.kind.get() };
        if let LazyKind::Vacant(vacant) = kind {
            let value = crate::env::get_contract_storage::<T>(vacant.key)
                .expect("couldn't find contract storage entry")
                .expect("couldn't properly decode contract storage entry");
            *kind = LazyKind::Occupied(OccupiedLazy::new(value));
        }
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
    #[must_use]
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
    #[must_use]
    pub fn get_mut(&mut self) -> &mut T {
        self.load_value_lazily();
        match self.kind_mut() {
            LazyKind::Vacant(_) => panic!("expect occupied lazy here"),
            LazyKind::Occupied(occupied) => &mut occupied.value,
        }
    }
}

impl<T> From<T> for Lazy<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Default for Lazy<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> core::cmp::PartialEq for Lazy<T>
where
    T: PartialEq + scale::Decode,
{
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.get(), other.get())
    }
}

impl<T> core::cmp::Eq for Lazy<T> where T: Eq + scale::Decode {}

impl<T> core::cmp::PartialOrd for Lazy<T>
where
    T: PartialOrd + scale::Decode,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(self.get(), other.get())
    }
    fn lt(&self, other: &Self) -> bool {
        PartialOrd::lt(self.get(), other.get())
    }
    fn le(&self, other: &Self) -> bool {
        PartialOrd::le(self.get(), other.get())
    }
    fn ge(&self, other: &Self) -> bool {
        PartialOrd::ge(self.get(), other.get())
    }
    fn gt(&self, other: &Self) -> bool {
        PartialOrd::gt(self.get(), other.get())
    }
}

impl<T> core::cmp::Ord for Lazy<T>
where
    T: core::cmp::Ord + scale::Decode,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(self.get(), other.get())
    }
}

impl<T> core::fmt::Display for Lazy<T>
where
    T: scale::Decode + core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(self.get(), f)
    }
}

impl<T> core::hash::Hash for Lazy<T>
where
    T: core::hash::Hash + scale::Decode,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

impl<T> core::convert::AsRef<T> for Lazy<T>
where
    T: scale::Decode,
{
    fn as_ref(&self) -> &T {
        self.get()
    }
}

impl<T> core::convert::AsMut<T> for Lazy<T>
where
    T: scale::Decode,
{
    fn as_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

impl<T> ink_prelude::borrow::Borrow<T> for Lazy<T>
where
    T: scale::Decode,
{
    fn borrow(&self) -> &T {
        self.get()
    }
}

impl<T> ink_prelude::borrow::BorrowMut<T> for Lazy<T>
where
    T: scale::Decode,
{
    fn borrow_mut(&mut self) -> &mut T {
        self.get_mut()
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
#[derive(Debug)]
pub enum LazyKind<T> {
    /// A true lazy storage entity that loads its contract storage value upon first use.
    Vacant(VacantLazy),
    /// An already loaded eager lazy storage entity.
    Occupied(OccupiedLazy<T>),
}

/// The lazy storage entity is in a lazy state.
#[derive(Debug)]
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
#[derive(Debug)]
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
