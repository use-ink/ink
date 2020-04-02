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

/// The lazy storage entry can be in either of two states.
///
/// - Vacant: No value has been loaded and the next access to the lazy cell
///           will lazily load and decode the value from contract storage.
/// - Occupied: There already is a value that has been loaded or that has been
///             simply set upon initialization.
#[derive(Debug)]
pub enum LazyCellEntry<T> {
    /// A true lazy storage entity that loads its contract storage value upon first use.
    Vacant(VacantEntry),
    /// An already loaded eager lazy storage entity.
    Occupied(Entry<T>),
}

/// The lazy storage entity is in a lazy state.
#[derive(Debug)]
pub struct VacantEntry {
    /// The key to load the value from contract storage upon first use.
    pub key: Key,
}

impl VacantEntry {
    /// Creates a new truly lazy storage entity for the given key.
    pub fn new(key: Key) -> Self {
        Self { key }
    }
}

/// A lazy storage entity.
///
/// This loads its value from storage upon first use.
///
/// # Note
///
/// Use this if the storage field doesn't need to be loaded in some or most cases.
#[derive(Debug)]
pub struct LazyCell<T> {
    // SAFETY: We use `UnsafeCell` instead of `RefCell` because
    //         the intended use-case is to hand out references (`&` and `&mut`)
    //         to the callers of `Lazy`. This cannot be done without `unsafe`
    //         code even with `RefCell`. Also `RefCell` has a larger footprint
    //         and has additional overhead that we can avoid by the interface
    //         and the fact that ink! code is always run single-threaded.
    //         Being efficient is important here because this is intended to be
    //         a low-level primitive with lots of dependencies.
    kind: UnsafeCell<LazyCellEntry<T>>,
}

impl<T> StorageFootprint for LazyCell<T>
where
    T: StorageFootprint,
{
    type Value = <T as StorageFootprint>::Value;
    const VALUE: u64 = <T as StorageFootprint>::VALUE;
}

impl<T> PullForward for LazyCell<T>
where
    T: StorageFootprint,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        Self::lazy(ptr.next_for::<T>())
    }
}

impl<T> PushForward for LazyCell<T>
where
    T: PushForward + StorageFootprint,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        // We skip pushing to contract storage if we are still in unloaded form.
        if let LazyCellEntry::Occupied(occupied) = self.kind() {
            if !occupied.is_mutated() {
                // Don't sync with storage if the value has not been mutated.
                return
            }
            match occupied.value() {
                Some(value) => <T as PushForward>::push_forward(value, ptr),
                None => {
                    // TODO: Find better and more general clean-up strategy with
                    //       the help of the proposed subtrie API.
                    let footprint = <T as StorageFootprint>::VALUE;
                    if footprint >= 32 {
                        panic!("cannot clean up more than 32 cells at once")
                    }
                    let key = ptr.next_for::<T>();
                    // Clean up storage associated with the value.
                    for n in 0..footprint {
                        crate::env::clear_contract_storage(key + n)
                    }
                }
            }
        }
    }
}

impl<T> ClearForward for LazyCell<T>
where
    T: ClearForward,
{
    fn clear_forward(&self, _ptr: &mut KeyPtr) {
        // Not implemented because at this point we are unsure whether
        // the whole clean-up traits are useful at all.
        todo!()
    }
}

impl<T> From<T> for LazyCell<T> {
    fn from(value: T) -> Self {
        Self::new(Some(value))
    }
}

impl<T> Default for LazyCell<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(Some(Default::default()))
    }
}

impl<T> LazyCell<T> {
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
            kind: UnsafeCell::new(LazyCellEntry::Occupied(Entry::new(
                value.into(),
                EntryState::Mutated,
            ))),
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
            kind: UnsafeCell::new(LazyCellEntry::Vacant(VacantEntry::new(key))),
        }
    }

    /// Returns a shared reference to the inner lazy kind.
    #[must_use]
    fn kind(&self) -> &LazyCellEntry<T> {
        // SAFETY: We just return a shared reference while the method receiver
        //         is a shared reference (&self) itself. So we respect normal
        //         Rust rules.
        unsafe { &*self.kind.get() }
    }
}

impl<T> LazyCell<T>
where
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
        let kind = unsafe { &mut *self.kind.get() };
        match kind {
            LazyCellEntry::Vacant(vacant) => {
                // Load the value from contract storage lazily.
                let mut key_ptr = KeyPtr::from(vacant.key);
                let value = <Option<T> as PullForward>::pull_forward(&mut key_ptr);
                let entry = Entry::new(value, EntryState::Mutated);
                *kind = LazyCellEntry::Occupied(entry);
                match kind {
                    LazyCellEntry::Vacant(_) => {
                        unreachable!("we just occupied the entry")
                    }
                    LazyCellEntry::Occupied(entry) => NonNull::from(entry),
                }
            }
            LazyCellEntry::Occupied(entry) => NonNull::from(entry),
        }
    }

    /// Returns a shared reference to the entry.
    fn get_entry(&self) -> &Entry<T> {
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
    fn get_entry_mut(&mut self) -> &mut Entry<T> {
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
        self.get_entry().value().into()
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
        self.get_entry_mut().value_mut().into()
    }
}
