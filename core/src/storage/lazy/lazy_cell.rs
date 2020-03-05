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

use super::super::{
    ClearForward,
    KeyPtr,
    PullForward,
    PushForward,
    StorageSize,
    StorageFootprint,
};
use core::cell::UnsafeCell;
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
    Occupied(OccupiedEntry<T>),
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

/// An already loaded or otherwise occupied eager lazy storage entity.
#[derive(Debug)]
pub struct OccupiedEntry<T> {
    /// The loaded value.
    pub value: Option<T>,
}

impl<T> OccupiedEntry<T> {
    /// Creates a new eager lazy storage entity with the given value.
    pub fn new(value: Option<T>) -> Self {
        Self { value }
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

impl<T> StorageSize for LazyCell<T>
where
    T: StorageSize,
{
    const SIZE: u64 = <T as StorageSize>::SIZE;
}

impl<T> StorageFootprint for LazyCell<T>
where
    T: StorageFootprint,
{
    type Value = <T as StorageFootprint>::Value;
}

impl<T> PullForward for LazyCell<T>
where
    T: StorageSize,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        Self::lazy(ptr.next_for::<T>())
    }
}

impl<T> PushForward for LazyCell<T>
where
    T: PushForward,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        // We skip pushing to contract storage if we are still in unloaded form.
        if let LazyCellEntry::Occupied(occupied) = self.kind() {
            if let Some(value) = &occupied.value {
                <T as PushForward>::push_forward(value, ptr)
            }
        }
    }
}

impl<T> ClearForward for LazyCell<T>
where
    T: ClearForward,
{
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        // We skip clear contract storage if we are still in unloaded form.
        if let LazyCellEntry::Occupied(occupied) = self.kind() {
            if let Some(value) = &occupied.value {
                <T as ClearForward>::clear_forward(value, ptr)
            }
        }
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
            kind: UnsafeCell::new(LazyCellEntry::Occupied(OccupiedEntry::new(
                value.into(),
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

    /// Returns an exclusive reference to the inner lazy kind.
    #[must_use]
    fn kind_mut(&mut self) -> &mut LazyCellEntry<T> {
        // SAFETY: We just return an exclusive reference while the method receiver
        //         is an exclusive reference (&mut self) itself. So we respect normal
        //         Rust rules.
        unsafe { &mut *self.kind.get() }
    }
}

impl<T> LazyCell<T>
where
    T: StorageSize + PullForward,
{
    /// Loads the value lazily from contract storage.
    ///
    /// # Note
    ///
    /// - After a successful call there will be an occupied value in the entry.
    /// - Does nothing if value has already been loaded.
    ///
    /// # Panics
    ///
    /// If a value has been loaded that failed to decode into `T`.
    fn load_value_lazily(&self) {
        // SAFETY: This is critical because we mutably access the entry.
        //         However, we mutate the entry only if it is vacant.
        //         If the entry is occupied by a value we return early.
        //         This way we do not invalidate pointers to this value.
        let kind = unsafe { &mut *self.kind.get() };
        if let LazyCellEntry::Vacant(vacant) = kind {
            let value =
                <Option<T> as PullForward>::pull_forward(&mut KeyPtr::from(vacant.key));
            *kind = LazyCellEntry::Occupied(OccupiedEntry::new(value));
        }
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
        self.load_value_lazily();
        match self.kind() {
            LazyCellEntry::Vacant(_) => unreachable!("assumed occupied value here"),
            LazyCellEntry::Occupied(occupied) => occupied.value.as_ref(),
        }
    }

    /// Returns an exclusive reference to the value.
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
        self.load_value_lazily();
        match self.kind_mut() {
            LazyCellEntry::Vacant(_) => unreachable!("assumed occupied value here"),
            LazyCellEntry::Occupied(occupied) => occupied.value.as_mut(),
        }
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
