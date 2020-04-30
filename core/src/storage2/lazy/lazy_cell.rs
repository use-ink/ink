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
    Entry,
    EntryState,
};
use crate::storage2::{
    traits2::{
        clear_spread_root_opt,
        pull_spread_root_opt,
        KeyPtr as KeyPtr2,
        SpreadLayout,
    },
    ClearForward,
    KeyPtr,
    PullForward,
    PushForward,
    StorageFootprint,
};
use core::{
    cell::UnsafeCell,
    ptr::NonNull,
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
pub struct LazyCell<T>
where
    T: SpreadLayout,
{
    /// The key to lazily load the value from.
    ///
    /// # Note
    ///
    /// This can be `None` on contract initialization where a `LazyCell` is
    /// normally initialized given a concrete value.
    key: Option<Key>,
    /// The low-level cache for the lazily loaded storage value.
    ///
    /// # Safety (Dev)
    ///
    /// We use `UnsafeCell` instead of `RefCell` because
    /// the intended use-case is to hand out references (`&` and `&mut`)
    /// to the callers of `Lazy`. This cannot be done without `unsafe`
    /// code even with `RefCell`. Also `RefCell` has a larger memory footprint
    /// and has additional overhead that we can avoid by the interface
    /// and the fact that ink! code is always run single-threaded.
    /// Being efficient is important here because this is intended to be
    /// a low-level primitive with lots of dependencies.
    cache: UnsafeCell<Entry<T>>,
}

impl<T> Drop for LazyCell<T>
where
    T: SpreadLayout,
{
    fn drop(&mut self) {
        if let Some(key) = self.key() {
            clear_spread_root_opt(self.value(), key)
        }
    }
}

impl<T> SpreadLayout for LazyCell<T>
where
    T: SpreadLayout,
{
    const FOOTPRINT: u64 = <T as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr2) -> Self {
        Self::lazy(ptr.next_for::<T>())
    }

    fn push_spread(&self, ptr: &mut KeyPtr2) {
        SpreadLayout::push_spread(self.entry(), ptr)
    }

    fn clear_spread(&self, ptr: &mut KeyPtr2) {
        SpreadLayout::clear_spread(self.entry(), ptr)
    }
}

// # Developer Note
//
// Implementing PackedLayout for LazyCell is not useful since that would
// potentially allow overlapping distinct LazyCell instances by pulling
// from the same underlying storage cell.
//
// If a user wants a packed LazyCell they can instead pack its inner type.

impl<T> StorageFootprint for LazyCell<T>
where
    T: SpreadLayout,
    T: StorageFootprint,
{
    const VALUE: u64 = <T as StorageFootprint>::VALUE;
}

impl<T> PullForward for LazyCell<T>
where
    T: SpreadLayout,
    T: StorageFootprint,
{
    fn pull_forward(_ptr: &mut KeyPtr) -> Self {
        unimplemented!("deprecated trait")
    }
}

impl<T> PushForward for LazyCell<T>
where
    T: SpreadLayout,
    T: PushForward + StorageFootprint,
{
    fn push_forward(&self, _ptr: &mut KeyPtr) {
        unimplemented!("deprecated trait")
    }
}

impl<T> ClearForward for LazyCell<T>
where
    T: SpreadLayout,
    T: ClearForward + StorageFootprint,
{
    fn clear_forward(&self, _ptr: &mut KeyPtr) {
        unimplemented!("deprecated trait")
    }
}

impl<T> From<T> for LazyCell<T>
where
    T: SpreadLayout,
{
    fn from(value: T) -> Self {
        Self::new(Some(value))
    }
}

impl<T> Default for LazyCell<T>
where
    T: Default + SpreadLayout,
{
    fn default() -> Self {
        Self::new(Some(Default::default()))
    }
}

impl<T> LazyCell<T>
where
    T: SpreadLayout,
{
    /// Creates an already populated lazy storage cell.
    ///
    /// # Note
    ///
    /// Since this already has a value it will never actually load from
    /// the contract storage.
    #[must_use]
    pub fn new<I>(value: I) -> Self
    where
        I: Into<Option<T>>,
    {
        Self {
            key: None,
            cache: UnsafeCell::new(Entry::new(value.into(), EntryState::Mutated)),
        }
    }

    /// Creates a lazy storage cell for the given key.
    ///
    /// # Note
    ///
    /// This will actually lazily load from the associated storage cell
    /// upon access.
    #[must_use]
    pub fn lazy(key: Key) -> Self {
        Self {
            key: Some(key),
            cache: UnsafeCell::new(Entry::new(None, EntryState::Preserved)),
        }
    }

    /// Returns the lazy key if any.
    ///
    /// # Note
    ///
    /// The key is `None` if the `LazyCell` has been initialized as a value.
    /// This generally only happens in ink! constructors.
    fn key(&self) -> Option<&Key> {
        self.key.as_ref()
    }

    /// Returns the cached value if any.
    ///
    /// # Note
    ///
    /// The cached value is `None` if the `LazyCell` has been initialized
    /// as lazy and has not yet loaded any value. This generally only happens
    /// in ink! messages.
    fn value(&self) -> Option<&T> {
        self.entry().value().into()
    }

    /// Returns the cached entry.
    fn entry(&self) -> &Entry<T> {
        unsafe { &*self.cache.get() }
    }
}

impl<T> LazyCell<T>
where
    T: SpreadLayout,
    T: StorageFootprint + PullForward,
{
    /// Loads the storage entry.
    ///
    /// Tries to load the entry from cache and falls back to lazily load the
    /// entry from the contract storage.
    ///
    /// # Panics
    ///
    /// Upon lazy loading if the lazy cell is in a state that forbids lazy loading.
    unsafe fn load_through_cache(&self) -> NonNull<Entry<T>> {
        // SAFETY: This is critical because we mutably access the entry.
        //         However, we mutate the entry only if it is vacant.
        //         If the entry is occupied by a value we return early.
        //         This way we do not invalidate pointers to this value.
        #[allow(unused_unsafe)]
        let cache = unsafe { &mut *self.cache.get() };
        if cache.value().is_none() {
            // Load value from storage and then return the cached entry.
            let key = self.key.expect("key required for lazy loading");
            let value = pull_spread_root_opt::<T>(&key);
            cache.put(value);
            cache.set_state(EntryState::Mutated);
        }
        NonNull::from(cache)
    }

    /// Returns a shared reference to the entry.
    fn load_entry(&self) -> &Entry<T> {
        // SAFETY: We load the entry either from cache of from contract storage.
        //
        //         This is safe because we are just returning a shared reference
        //         from within a `&self` method. This also cannot change the
        //         loaded value and thus cannot change the `mutate` flag of the
        //         entry. Aliases using this method are safe since ink! is
        //         single-threaded.
        unsafe { &*self.load_through_cache().as_ptr() }
    }

    /// Returns an exclusive reference to the entry.
    fn load_entry_mut(&mut self) -> &mut Entry<T> {
        // SAFETY: We load the entry either from cache of from contract storage.
        //
        //         This is safe because we are just returning a shared reference
        //         from within a `&self` method. This also cannot change the
        //         loaded value and thus cannot change the `mutate` flag of the
        //         entry. Aliases using this method are safe since ink! is
        //         single-threaded.
        let entry = unsafe { &mut *self.load_through_cache().as_ptr() };
        entry.set_state(EntryState::Mutated);
        entry
    }

    /// Returns a shared reference to the value.
    ///
    /// # Note
    ///
    /// This eventually lazily loads the value from the contract storage.
    ///
    /// # Panics
    ///
    /// If decoding the loaded value to `T` failed.
    #[must_use]
    pub fn get(&self) -> Option<&T> {
        self.load_entry().value().into()
    }

    /// Returns a shared reference to the value.
    ///
    /// # Note
    ///
    /// This eventually lazily loads the value from the contract storage.
    ///
    /// # Panics
    ///
    /// If decoding the loaded value to `T` failed.
    #[must_use]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.load_entry_mut().value_mut().into()
    }
}
