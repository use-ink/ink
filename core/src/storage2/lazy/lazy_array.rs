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
use crate::storage2::traits::{
    clear_packed_root,
    pull_packed_root_opt,
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use core::{
    cell::UnsafeCell,
    fmt,
    fmt::Debug,
    mem,
    ptr::NonNull,
};
use generic_array::{
    typenum::{
        UInt,
        UTerm,
        Unsigned,
        B0,
        B1,
    },
    ArrayLength,
    GenericArray,
};
use ink_primitives::Key;

/// The index type used in the lazy storage chunk.
pub type Index = u32;

/// Utility trait for helping with lazy array construction.
pub trait LazyArrayLength<T>:
    ArrayLength<UnsafeCell<Option<Entry<T>>>> + Unsigned
{
}
impl<T> LazyArrayLength<T> for UTerm {}
impl<T, N: ArrayLength<UnsafeCell<Option<Entry<T>>>>> LazyArrayLength<T> for UInt<N, B0> {}
impl<T, N: ArrayLength<UnsafeCell<Option<Entry<T>>>>> LazyArrayLength<T> for UInt<N, B1> {}

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
    cached_entries: EntryArray<T, N>,
}

struct DebugEntryArray<'a, T, N>(&'a EntryArray<T, N>)
where
    T: Debug,
    N: LazyArrayLength<T>;

impl<'a, T, N> Debug for DebugEntryArray<'a, T, N>
where
    T: Debug,
    N: LazyArrayLength<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entries(self.0.iter().enumerate().filter_map(|(key, entry)| {
                match entry {
                    Some(entry) => Some((key, entry)),
                    None => None,
                }
            }))
            .finish()
    }
}

impl<T, N> Debug for LazyArray<T, N>
where
    T: Debug,
    N: LazyArrayLength<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LazyArray")
            .field("key", &self.key)
            .field("cached_entries", &DebugEntryArray(&self.cached_entries))
            .finish()
    }
}

#[test]
fn debug_impl_works() {
    use generic_array::typenum::U4;
    let mut larray = <LazyArray<i32, U4>>::new();
    // Empty imap.
    assert_eq!(
        format!("{:?}", &larray),
        "LazyArray { key: None, cached_entries: {} }",
    );
    // Filled imap.
    larray.put(0, Some(1));
    larray.put(2, Some(2));
    larray.put(3, None);
    assert_eq!(
        format!("{:?}", &larray),
        "LazyArray { \
            key: None, \
            cached_entries: {\
                0: Entry { \
                    value: Some(1), \
                    state: Mutated \
                }, \
                2: Entry { \
                    value: Some(2), \
                    state: Mutated \
                }, \
                3: Entry { \
                    value: None, \
                    state: Mutated \
                }\
            } \
        }",
    );
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
    entries: GenericArray<UnsafeCell<Option<Entry<T>>>, N>,
}

#[derive(Debug)]
pub struct EntriesIter<'a, T> {
    iter: core::slice::Iter<'a, UnsafeCell<Option<Entry<T>>>>,
}

impl<'a, T> EntriesIter<'a, T> {
    pub fn new<N>(entry_array: &'a EntryArray<T, N>) -> Self
    where
        N: LazyArrayLength<T>,
    {
        Self {
            iter: entry_array.entries.iter(),
        }
    }
}

impl<'a, T> Iterator for EntriesIter<'a, T> {
    type Item = &'a Option<Entry<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|cell| unsafe { &*cell.get() })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.iter.count()
    }
}

impl<'a, T> DoubleEndedIterator for EntriesIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|cell| unsafe { &*cell.get() })
    }
}

impl<'a, T> ExactSizeIterator for EntriesIter<'a, T> {}

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
    fn put(&self, at: Index, new_value: Option<T>) -> Option<T> {
        mem::replace(
            unsafe { &mut *self.entries.as_slice()[at as usize].get() },
            Some(Entry::new(new_value, EntryState::Mutated)),
        )
        .map(Entry::into_value)
        .flatten()
    }

    /// Inserts a new entry into the cache and returns an exclusive reference to it.
    unsafe fn insert_entry(&self, at: Index, new_entry: Entry<T>) -> NonNull<Entry<T>> {
        let entry: &mut Option<Entry<T>> =
            unsafe { &mut *UnsafeCell::get(&self.entries[at as usize]) };
        *entry = Some(new_entry);
        entry
            .as_mut()
            .map(NonNull::from)
            .expect("just inserted the entry")
    }

    /// Returns an exclusive reference to the entry at the given index if any.
    unsafe fn get_entry_mut(&self, at: Index) -> Option<&mut Entry<T>> {
        if at >= Self::capacity() {
            return None
        }
        (&mut *UnsafeCell::get(&self.entries[at as usize])).as_mut()
    }

    /// Returns an iterator that yields shared references to all cached entries.
    pub fn iter(&self) -> EntriesIter<T> {
        EntriesIter::new(self)
    }
}

impl<T, N> LazyArray<T, N>
where
    T: PackedLayout,
    N: LazyArrayLength<T>,
{
    /// Clears the underlying storage of the entry at the given index.
    ///
    /// # Safety
    ///
    /// For performance reasons this does not synchronize the lazy array's
    /// memory-side cache which invalidates future accesses the cleared entry.
    /// Care should be taken when using this API.
    ///
    /// The general use of this API is to streamline `Drop` implementations of
    /// high-level abstractions that build upon this low-level data strcuture.
    pub fn clear_packed_at(&self, index: Index) {
        let root_key = self.key.expect("cannot clear in lazy state");
        if <T as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP {
            // We need to load the entity before we remove its associated contract storage
            // because it requires a deep clean-up which propagates clearing to its fields,
            // for example in the case of `T` being a `storage::Box`.
            let entity = self.get(index).expect("cannot clear a non existing entity");
            clear_packed_root::<T>(&entity, &root_key);
        } else {
            // The type does not require deep clean-up so we can simply clean-up
            // its associated storage cell and be done without having to load it first.
            crate::env::clear_contract_storage(root_key);
        }
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
            cached_entries: Default::default(),
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
        &self.cached_entries
    }

    /// Puts a new value into the given indexed slot.
    ///
    /// # Note
    ///
    /// Use [`LazyArray::put_get`]`(None)` to remove an element.
    pub fn put(&mut self, at: Index, new_value: Option<T>) {
        self.cached_entries().put(at, new_value);
    }
}

impl<T, N> SpreadLayout for LazyArray<T, N>
where
    T: PackedLayout,
    N: LazyArrayLength<T>,
{
    const FOOTPRINT: u64 = <N as Unsigned>::U64;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Self {
            key: Some(ptr.next_for::<Self>()),
            cached_entries: EntryArray::new(),
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        let offset_key = ptr.next_for::<Self>();
        for (index, entry) in self.cached_entries().iter().enumerate() {
            if let Some(entry) = entry {
                let root_key = offset_key + index as u64;
                entry.push_packed_root(&root_key);
            }
        }
    }

    #[inline]
    fn clear_spread(&self, _ptr: &mut KeyPtr) {
        // Low-level lazy abstractions won't perform automated clean-up since
        // they generally are not aware of their entire set of associated
        // elements. The high-level abstractions that build upon them are
        // responsible for cleaning up.
    }
}

impl<T, N> LazyArray<T, N>
where
    N: LazyArrayLength<T>,
{
    /// Returns the offset key for the given index if not out of bounds.
    pub fn key_at(&self, at: Index) -> Option<Key> {
        if at >= Self::capacity() {
            return None
        }
        self.key.map(|key| key + at as u64)
    }
}

impl<T, N> LazyArray<T, N>
where
    T: PackedLayout,
    N: LazyArrayLength<T>,
{
    /// Loads the entry at the given index.
    ///
    /// Tries to load the entry from cache and falls back to lazily load the
    /// entry from the contract storage.
    fn load_through_cache(&self, at: Index) -> NonNull<Entry<T>> {
        assert!(at < Self::capacity(), "index is out of bounds");
        match unsafe { self.cached_entries.get_entry_mut(at) } {
            Some(entry) => {
                // Load value from cache.
                NonNull::from(entry)
            }
            None => {
                // Load value from storage and put into cache.
                // Then load value from cache.
                let value = self
                    .key_at(at)
                    .map(|key| pull_packed_root_opt::<T>(&key))
                    .unwrap_or(None);
                let entry = Entry::new(value, EntryState::Preserved);
                unsafe { self.cached_entries.insert_entry(at, entry) }
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
        unsafe { &*self.load_through_cache(at).as_ptr() }
            .value()
            .into()
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
        self.load_through_cache_mut(at).put(None)
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
        loaded_a.replace_state(EntryState::Mutated);
        loaded_b.replace_state(EntryState::Mutated);
        core::mem::swap(loaded_a.value_mut(), loaded_b.value_mut());
    }
}
