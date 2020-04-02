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
    KeyPtr,
    PullForward,
    PushForward,
    SaturatingStorage,
    StorageFootprint,
    StorageFootprintOf,
};
use core::{
    cell::UnsafeCell,
    mem,
    ops::Mul,
    ptr::NonNull,
};
use generic_array::{
    ArrayLength,
    GenericArray,
};
use ink_primitives::Key;
use typenum::{
    Prod,
    UInt,
    UTerm,
    Unsigned,
    B0,
    B1,
};

/// The index type used in the lazy storage chunk.
pub type Index = u32;

/// Utility trait for helping with lazy array construction.
pub trait LazyArrayLength<T>: ArrayLength<Option<Entry<T>>> + Unsigned {}
impl<T> LazyArrayLength<T> for UTerm {}
impl<T, N: ArrayLength<Option<Entry<T>>>> LazyArrayLength<T> for UInt<N, B0> {}
impl<T, N: ArrayLength<Option<Entry<T>>>> LazyArrayLength<T> for UInt<N, B1> {}

/// A lazy storage array that spans over N storage cells.
///
/// Storage data structure to emulate storage arrays: `[T; N]`.
///
/// # Note
///
/// Computes operations on the underlying N storage cells in a lazy fashion.
/// Due to the size constraints the `LazyArray` is generally more efficient
/// than the [`LazyMap`](`super::LazyIndexMap`) for most use cases with limited elements.
///
/// This is mainly used as low-level storage primitives by other high-level
/// storage primitives in order to manage the contract storage for a whole
/// chunk of storage cells.
#[derive(Debug)]
pub struct LazyArray<T, N>
where
    N: LazyArrayLength<T>,
{
    /// The offset key for the N cells.
    ///
    /// If the lazy chunk has been initialized during contract initialization
    /// the key will be `None` since there won't be a storage region associated
    /// to the lazy chunk which prevents it from lazily loading elements. This,
    /// however, is only checked at contract runtime. We might incorporate
    /// compile-time checks for this particular use case later on.
    key: Option<Key>,
    /// The subset of currently cached entries of the lazy storage chunk.
    ///
    /// An entry is cached as soon as it is loaded or written.
    cached_entries: UnsafeCell<EntryArray<T, N>>,
}

/// Returns the capacity for an array with the given array length.
fn array_capacity<T, N>() -> u32
where
    N: LazyArrayLength<T>,
{
    <N as Unsigned>::U32
}

/// The underlying array cache for the [`LazyArray`].
#[derive(Debug)]
pub struct EntryArray<T, N>
where
    N: LazyArrayLength<T>,
{
    /// The cache entries of the entry array.
    entries: GenericArray<Option<Entry<T>>, N>,
}

impl<T, N> EntryArray<T, N>
where
    N: LazyArrayLength<T>,
{
    /// Creates a new entry array cache.
    pub fn new() -> Self {
        Self {
            entries: Default::default(),
        }
    }
}

impl<T, N> Default for EntryArray<T, N>
where
    N: LazyArrayLength<T>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, N> EntryArray<T, N>
where
    N: LazyArrayLength<T>,
{
    /// Returns the constant capacity of the lazy array.
    #[inline]
    pub fn capacity() -> u32 {
        array_capacity::<T, N>()
    }

    /// Puts the the new value into the indexed slot and
    /// returns the old value if any.
    fn put(&mut self, at: Index, new_value: Option<T>) -> Option<T> {
        mem::replace(
            &mut self.entries.as_mut_slice()[at as usize],
            Some(Entry::new(new_value, EntryState::Mutated)),
        )
        .map(Entry::into_value)
        .flatten()
    }

    /// Inserts a new entry into the cache and returns an exclusive reference to it.
    fn insert_entry(&mut self, at: Index, entry: Entry<T>) -> &mut Entry<T> {
        self.entries[at as usize] = Some(entry);
        let entry: Option<&mut Entry<T>> = (&mut self.entries[at as usize]).into();
        entry.expect("just inserted the entry")
    }

    /// Returns an exclusive reference to the entry at the given index if any.
    fn get_entry_mut(&mut self, at: Index) -> Option<&mut Entry<T>> {
        if at >= Self::capacity() {
            return None
        }
        self.entries[at as usize].as_mut()
    }
}

impl<T, N> Default for LazyArray<T, N>
where
    N: LazyArrayLength<T>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, N> LazyArray<T, N>
where
    N: LazyArrayLength<T>,
{
    /// Creates a new empty lazy array.
    ///
    /// # Note
    ///
    /// A lazy array created this way cannot be used to load from the contract storage.
    /// All operations that directly or indirectly load from storage will panic.
    pub fn new() -> Self {
        Self {
            key: None,
            cached_entries: UnsafeCell::new(Default::default()),
        }
    }

    /// Returns the constant capacity of the lazy array.
    #[inline]
    pub fn capacity() -> u32 {
        array_capacity::<T, N>()
    }

    /// Returns the offset key of the lazy array if any.
    pub fn key(&self) -> Option<&Key> {
        self.key.as_ref()
    }

    /// Returns a shared reference to the underlying cached entries.
    ///
    /// # Safety
    ///
    /// This operation is safe since it returns a shared reference from
    /// a `&self` which is viable in safe Rust.
    fn cached_entries(&self) -> &EntryArray<T, N> {
        unsafe { &*self.cached_entries.get() }
    }

    /// Returns an exclusive reference to the underlying cached entries.
    ///
    /// # Safety
    ///
    /// This operation is safe since it returns an exclusive reference from
    /// a `&mut self` which is viable in safe Rust.
    fn cached_entries_mut(&mut self) -> &mut EntryArray<T, N> {
        unsafe { &mut *self.cached_entries.get() }
    }

    /// Puts a new value into the given indexed slot.
    ///
    /// # Note
    ///
    /// Use [`LazyArray::put_get`]`(None)` to remove an element.
    pub fn put(&mut self, at: Index, new_value: Option<T>) {
        self.cached_entries_mut().put(at, new_value);
    }
}

impl<T, N> StorageFootprint for LazyArray<T, N>
where
    T: StorageFootprint,
    N: LazyArrayLength<T>,
    StorageFootprintOf<T>: Mul<N>,
    Prod<StorageFootprintOf<T>, N>: Unsigned,
{
    type Value = Prod<StorageFootprintOf<T>, N>;
    const VALUE: u64 = <N as Unsigned>::U64;
}

impl<T, N> PullForward for LazyArray<T, N>
where
    Self: StorageFootprint,
    N: LazyArrayLength<T>,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        Self {
            key: Some(ptr.next_for::<Self>()),
            cached_entries: UnsafeCell::new(EntryArray::new()),
        }
    }
}

impl<T, N> PushForward for LazyArray<T, N>
where
    T: StorageFootprint + SaturatingStorage + PushForward,
    N: LazyArrayLength<T>,
    Self: StorageFootprint,
    <Self as StorageFootprint>::Value: Unsigned,
    <T as StorageFootprint>::Value: Unsigned,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        let offset_key = ptr.next_for::<Self>();
        for (index, entry) in self.cached_entries().entries.iter().enumerate() {
            if let Some(entry) = entry {
                if !entry.is_mutated() {
                    continue
                }
                let footprint = <<T as StorageFootprint>::Value as Unsigned>::to_u64();
                let key = offset_key + (index as u64 * footprint);
                match entry.value() {
                    Some(value) => {
                        // Update associated storage entries.
                        <T as PushForward>::push_forward(value, &mut KeyPtr::from(key))
                    }
                    None => {
                        // Clean-up associated storage entries.
                        if footprint > 32 {
                            // Bail out if footprint is too big.
                            //
                            // TODO:
                            // - Use compile-time solution to prevent situations like these.
                            //   This should be simple now since we are using `typenum` instead
                            //   of associated constants.
                            return
                        }
                        use crate::env;
                        for i in 0..footprint as u32 {
                            env::clear_contract_storage(key + i);
                        }
                    }
                }
            }
        }
    }
}

impl<T, N> LazyArray<T, N>
where
    T: StorageFootprint,
    <T as StorageFootprint>::Value: Unsigned,
    N: LazyArrayLength<T>,
{
    /// Returns the offset key for the given index if not out of bounds.
    pub fn key_at(&self, at: Index) -> Option<Key> {
        if at >= Self::capacity() {
            return None
        }
        self.key.map(|key| {
            key + ((at as u64) * <<T as StorageFootprint>::Value as Unsigned>::to_u64())
        })
    }
}

impl<T, N> LazyArray<T, N>
where
    T: StorageFootprint + PullForward,
    <T as StorageFootprint>::Value: Unsigned,
    N: LazyArrayLength<T>,
{
    /// Loads the entry at the given index.
    ///
    /// Tries to load the entry from cache and falls back to lazily load the
    /// entry from the contract storage.
    ///
    /// # Panics
    ///
    /// - If the lazy array is in a state that forbids lazy loading.
    /// - If the given index is out of bounds.
    fn load_through_cache(&self, at: Index) -> NonNull<Entry<T>> {
        assert!(at < Self::capacity(), "index is out of bounds");
        let cached_entries = unsafe { &mut *self.cached_entries.get() };
        match cached_entries.get_entry_mut(at) {
            Some(entry) => {
                // Load value from cache.
                NonNull::from(entry)
            }
            None => {
                // Load value from storage and put into cache.
                // Then load value from cache.
                let key = self.key_at(at).expect("cannot load lazily in this state");
                let value =
                    <Option<T> as PullForward>::pull_forward(&mut KeyPtr::from(key));
                let entry = Entry::new(value, EntryState::Preserved);
                NonNull::from(cached_entries.insert_entry(at, entry))
            }
        }
    }

    /// Loads the entry at the given index.
    ///
    /// Tries to load the entry from cache and falls back to lazily load the
    /// entry from the contract storage.
    ///
    /// # Panics
    ///
    /// - If the lazy array is in a state that forbids lazy loading.
    /// - If the given index is out of bounds.
    fn load_through_cache_mut(&mut self, index: Index) -> &mut Entry<T> {
        // SAFETY:
        // Returning a `&mut Entry<T>` from within a `&mut self` function
        // won't allow creating aliasing between exclusive references.
        unsafe { &mut *self.load_through_cache(index).as_ptr() }
    }

    /// Returns a shared reference to the element at the given index if any.
    ///
    /// # Note
    ///
    /// This operation eventually loads from contract storage.
    ///
    /// # Panics
    ///
    /// If the given index is out of bounds.
    pub fn get(&self, at: Index) -> Option<&T> {
        unsafe { &*self.load_through_cache(at).as_ptr() }.value().into()
    }

    /// Returns an exclusive reference to the element at the given index if any.
    ///
    /// # Note
    ///
    /// This operation eventually loads from contract storage.
    ///
    /// # Panics
    ///
    /// If the given index is out of bounds.
    pub fn get_mut(&mut self, at: Index) -> Option<&mut T> {
        self.load_through_cache_mut(at).value_mut().into()
    }

    /// Removes the element at the given index and returns it if any.
    ///
    /// # Note
    ///
    /// This operation eventually loads from contract storage.
    ///
    /// # Panics
    ///
    /// If the given index is out of bounds.
    pub fn take(&mut self, at: Index) -> Option<T> {
        self.load_through_cache_mut(at).take_value()
    }

    /// Puts the new value into the indexed slot and returns the old value if any.
    ///
    /// # Note
    ///
    /// - This operation eventually loads from contract storage.
    /// - Prefer [`LazyArray::put`] if you are not interested in the old value.
    /// - Use [`LazyArray::put_get`]`(None)` to remove an element.
    ///
    /// # Panics
    ///
    /// If the given index is out of bounds.
    pub fn put_get(&mut self, at: Index, new_value: Option<T>) -> Option<T> {
        self.load_through_cache_mut(at).put(new_value)
    }

    /// Swaps the values at indices x and y.
    ///
    /// # Note
    ///
    /// This operation eventually loads from contract storage.
    ///
    /// # Panics
    ///
    /// If any of the given indices is out of bounds.
    pub fn swap(&mut self, a: Index, b: Index) {
        if a == b {
            // Bail out early if both indices are the same.
            return
        }
        assert!(a < Self::capacity(), "a is out of bounds");
        assert!(b < Self::capacity(), "b is out of bounds");
        let (loaded_a, loaded_b) =
            // SAFETY: The loaded `x` and `y` entries are distinct from each
            //         other guaranteed by the previous checks so they cannot
            //         alias.
            unsafe { (
                &mut *self.load_through_cache(a).as_ptr(),
                &mut *self.load_through_cache(b).as_ptr(),
            ) };
        if loaded_a.value().is_none() && loaded_b.value().is_none() {
            // Bail out since nothing has to be swapped if both values are `None`.
            return
        }
        // At this point at least one of the values is `Some` so we have to
        // perform the swap and set both entry states to mutated.
        loaded_a.set_state(EntryState::Mutated);
        loaded_b.set_state(EntryState::Mutated);
        core::mem::swap(loaded_a.value_mut(), loaded_b.value_mut());
    }
}
