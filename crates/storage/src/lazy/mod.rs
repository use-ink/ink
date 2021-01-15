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

//! Low-level collections and data structures to manage storage entities in the
//! persisted contract storage.
//!
//! Users should generally avoid using these collections directly in their
//! contracts and should instead adhere to the high-level collections found
//! in [`crate::collections`].
//! The low-level collections are mainly used as building blocks for internals
//! of other higher-level storage collections.
//!
//! These low-level collections are not aware of the elements they manage thus
//! extra care has to be taken when operating directly on them.

pub mod lazy_hmap;

mod cache_cell;
mod entry;
mod lazy_array;
mod lazy_cell;
mod lazy_imap;

use self::{
    cache_cell::CacheCell,
    entry::{
        EntryState,
        StorageEntry,
    },
};
#[doc(inline)]
pub use self::{
    lazy_array::{
        LazyArray,
        LazyArrayLength,
    },
    lazy_cell::LazyCell,
    lazy_hmap::LazyHashMap,
    lazy_imap::LazyIndexMap,
};
use crate::traits::{
    KeyPtr,
    SpreadLayout,
};
use ink_primitives::Key;

/// A lazy storage entity.
///
/// This loads its value from storage upon first use.
///
/// # Note
///
/// Use this if the storage field doesn't need to be loaded in some or most cases.
#[derive(Debug)]
pub struct Lazy<T>
where
    T: SpreadLayout,
{
    cell: LazyCell<T>,
}

#[cfg(feature = "std")]
const _: () = {
    use crate::traits::StorageLayout;
    use ink_metadata::layout::Layout;

    impl<T> StorageLayout for Lazy<T>
    where
        T: StorageLayout + SpreadLayout,
    {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            <T as StorageLayout>::layout(key_ptr)
        }
    }
};

impl<T> SpreadLayout for Lazy<T>
where
    T: SpreadLayout,
{
    const FOOTPRINT: u64 = <T as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Self {
            cell: <LazyCell<T> as SpreadLayout>::pull_spread(ptr),
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::push_spread(&self.cell, ptr)
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::clear_spread(&self.cell, ptr)
    }
}

impl<T> Lazy<T>
where
    T: SpreadLayout,
{
    /// Creates an eagerly populated lazy storage value.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self {
            cell: LazyCell::new(Some(value)),
        }
    }

    /// Creates a true lazy storage value for the given key.
    #[must_use]
    pub(crate) fn lazy(key: Key) -> Self {
        Self {
            cell: LazyCell::lazy(key),
        }
    }
}

impl<T> Lazy<T>
where
    T: SpreadLayout,
{
    /// Returns a shared reference to the lazily loaded value.
    ///
    /// # Note
    ///
    /// This loads the value from the contract storage if this did not happen before.
    ///
    /// # Panics
    ///
    /// If loading from contract storage failed.
    #[must_use]
    pub fn get(lazy: &Self) -> &T {
        lazy.cell.get().expect("encountered empty storage cell")
    }

    /// Returns an exclusive reference to the lazily loaded value.
    ///
    /// # Note
    ///
    /// This loads the value from the contract storage if this did not happen before.
    ///
    /// # Panics
    ///
    /// If loading from contract storage failed.
    #[must_use]
    pub fn get_mut(lazy: &mut Self) -> &mut T {
        lazy.cell.get_mut().expect("encountered empty storage cell")
    }

    /// Sets the value to `value`, without executing any reads.
    ///
    /// # Note
    ///
    /// No reads from contract storage will be executed.
    ///
    /// This method should be preferred over dereferencing or `get_mut`
    /// in case the returned value is of no interest to the caller.
    ///
    /// # Panics
    ///
    /// If accessing the inner value fails.
    #[inline]
    pub fn set(lazy: &mut Self, new_value: T) {
        lazy.cell.set(new_value);
    }
}

impl<T> From<T> for Lazy<T>
where
    T: SpreadLayout,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Default for Lazy<T>
where
    T: Default + SpreadLayout,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> core::cmp::PartialEq for Lazy<T>
where
    T: PartialEq + SpreadLayout,
{
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(Lazy::get(self), Lazy::get(other))
    }
}

impl<T> core::cmp::Eq for Lazy<T> where T: Eq + SpreadLayout {}

impl<T> core::cmp::PartialOrd for Lazy<T>
where
    T: PartialOrd + SpreadLayout,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(Lazy::get(self), Lazy::get(other))
    }
    fn lt(&self, other: &Self) -> bool {
        PartialOrd::lt(Lazy::get(self), Lazy::get(other))
    }
    fn le(&self, other: &Self) -> bool {
        PartialOrd::le(Lazy::get(self), Lazy::get(other))
    }
    fn ge(&self, other: &Self) -> bool {
        PartialOrd::ge(Lazy::get(self), Lazy::get(other))
    }
    fn gt(&self, other: &Self) -> bool {
        PartialOrd::gt(Lazy::get(self), Lazy::get(other))
    }
}

impl<T> core::cmp::Ord for Lazy<T>
where
    T: core::cmp::Ord + SpreadLayout,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(Lazy::get(self), Lazy::get(other))
    }
}

impl<T> core::fmt::Display for Lazy<T>
where
    T: core::fmt::Display + SpreadLayout,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(Lazy::get(self), f)
    }
}

impl<T> core::hash::Hash for Lazy<T>
where
    T: core::hash::Hash + SpreadLayout,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        Lazy::get(self).hash(state);
    }
}

impl<T> core::convert::AsRef<T> for Lazy<T>
where
    T: SpreadLayout,
{
    fn as_ref(&self) -> &T {
        Lazy::get(self)
    }
}

impl<T> core::convert::AsMut<T> for Lazy<T>
where
    T: SpreadLayout,
{
    fn as_mut(&mut self) -> &mut T {
        Lazy::get_mut(self)
    }
}

impl<T> ink_prelude::borrow::Borrow<T> for Lazy<T>
where
    T: SpreadLayout,
{
    fn borrow(&self) -> &T {
        Lazy::get(self)
    }
}

impl<T> ink_prelude::borrow::BorrowMut<T> for Lazy<T>
where
    T: SpreadLayout,
{
    fn borrow_mut(&mut self) -> &mut T {
        Lazy::get_mut(self)
    }
}

impl<T> core::ops::Deref for Lazy<T>
where
    T: SpreadLayout,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Lazy::get(self)
    }
}

impl<T> core::ops::DerefMut for Lazy<T>
where
    T: SpreadLayout,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        Lazy::get_mut(self)
    }
}
