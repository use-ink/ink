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

use super::{
    CacheCell,
    EntryState,
    StorageEntry,
};
use crate::traits::{
    clear_packed_root,
    pull_packed_root_opt,
    ExtKeyPtr,
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use core::{
    fmt,
    fmt::Debug,
    mem,
    ptr::NonNull,
};
use ink_primitives::Key;

/// The index type used in the lazy storage chunk.
pub type Index = u32;

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
pub struct LazyArray<T, const N: usize> {
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

#[cfg(feature = "std")]
const _: () = {
    use crate::traits::StorageLayout;
    use ink_metadata::layout::{
        ArrayLayout,
        CellLayout,
        Layout,
        LayoutKey,
    };
    use scale_info::TypeInfo;

    impl<T, const N: usize> StorageLayout for LazyArray<T, N>
    where
        T: TypeInfo + 'static,
    {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            let capacity = N as u32;
            Layout::Array(ArrayLayout::new(
                LayoutKey::from(key_ptr.advance_by(capacity as u64)),
                capacity,
                1,
                Layout::Cell(CellLayout::new::<T>(LayoutKey::from(
                    key_ptr.advance_by(0),
                ))),
            ))
        }
    }
};

struct DebugEntryArray<'a, T, const N: usize>(&'a EntryArray<T, N>)
where
    T: Debug;

impl<'a, T, const N: usize> Debug for DebugEntryArray<'a, T, N>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entries(
                self.0
                    .iter()
                    .enumerate()
                    .filter_map(|(key, entry)| entry.as_ref().map(|entry| (key, entry))),
            )
            .finish()
    }
}

impl<T, const N: usize> Debug for LazyArray<T, N>
where
    T: Debug,
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
    let mut larray = <LazyArray<i32, 4>>::new();
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
fn array_capacity<T, const N: usize>() -> u32 {
    N as u32
}

/// The underlying array cache for the [`LazyArray`].
#[derive(Debug)]
pub struct EntryArray<T, const N: usize> {
    /// The cache entries of the entry array.
    entries: [CacheCell<Option<StorageEntry<T>>>; N],
}

#[derive(Debug)]
pub struct EntriesIter<'a, T> {
    iter: core::slice::Iter<'a, CacheCell<Option<StorageEntry<T>>>>,
}

impl<'a, T> EntriesIter<'a, T> {
    pub fn new<const N: usize>(entry_array: &'a EntryArray<T, N>) -> Self {
        Self {
            iter: entry_array.entries.iter(),
        }
    }
}

impl<'a, T> Iterator for EntriesIter<'a, T> {
    type Item = &'a Option<StorageEntry<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|cell| cell.as_inner())
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
        self.iter.next_back().map(|cell| cell.as_inner())
    }
}

impl<'a, T> ExactSizeIterator for EntriesIter<'a, T> {}

impl<T, const N: usize> EntryArray<T, N> {
    /// Creates a new entry array cache.
    pub fn new() -> Self {
        Self {
            entries: array_init::array_init(|_| Default::default()),
        }
    }
}

impl<T, const N: usize> Default for EntryArray<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> EntryArray<T, N> {
    /// Returns the constant capacity of the lazy array.
    #[inline]
    pub fn capacity() -> u32 {
        array_capacity::<T, N>()
    }

    /// Puts the the new value into the indexed slot and
    /// returns the old value if any.
    fn put(&self, at: Index, new_value: Option<T>) -> Option<T> {
        mem::replace(
            unsafe { self.entries[at as usize].get_ptr().as_mut() },
            Some(StorageEntry::new(new_value, EntryState::Mutated)),
        )
        .map(StorageEntry::into_value)
        .flatten()
    }

    /// Inserts a new entry into the cache and returns an exclusive reference to it.
    unsafe fn insert_entry(
        &self,
        at: Index,
        new_entry: StorageEntry<T>,
    ) -> NonNull<StorageEntry<T>> {
        let entry: &mut Option<StorageEntry<T>> =
            &mut *CacheCell::get_ptr(&self.entries[at as usize]).as_ptr();
        *entry = Some(new_entry);
        entry
            .as_mut()
            .map(NonNull::from)
            .expect("just inserted the entry")
    }

    /// Returns an exclusive reference to the entry at the given index if any.
    unsafe fn get_entry_mut(&self, at: Index) -> Option<&mut StorageEntry<T>> {
        if at >= Self::capacity() {
            return None
        }
        (&mut *CacheCell::get_ptr(&self.entries[at as usize]).as_ptr()).as_mut()
    }

    /// Returns an iterator that yields shared references to all cached entries.
    pub fn iter(&self) -> EntriesIter<T> {
        EntriesIter::new(self)
    }
}

impl<T, const N: usize> LazyArray<T, N>
where
    T: PackedLayout,
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
    /// high-level abstractions that build upon this low-level data structure.
    pub fn clear_packed_at(&self, index: Index) {
        let root_key = self.key_at(index).expect("cannot clear in lazy state");
        if <T as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP {
            // We need to load the entity before we remove its associated contract storage
            // because it requires a deep clean-up which propagates clearing to its fields,
            // for example in the case of `T` being a `storage::Box`.
            let entity = self.get(index).expect("cannot clear a non existing entity");
            clear_packed_root::<T>(entity, &root_key);
        } else {
            // The type does not require deep clean-up so we can simply clean-up
            // its associated storage cell and be done without having to load it first.
            ink_env::clear_contract_storage(&root_key);
        }
    }
}

impl<T, const N: usize> Default for LazyArray<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> LazyArray<T, N> {
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

    /// Creates a new empty lazy array positioned at the given key.
    ///
    /// # Note
    ///
    /// This constructor is private and should never need to be called from
    /// outside this module. It is used to construct a lazy array from a
    /// key that is only useful upon a contract call.
    /// Use [`LazyArray::new`] for construction during contract initialization.
    fn lazy(key: Key) -> Self {
        Self {
            key: Some(key),
            cached_entries: Default::default(),
        }
    }

    /// Returns the constant capacity of the lazy array.
    #[inline]
    pub fn capacity(&self) -> u32 {
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

impl<T, const N: usize> SpreadLayout for LazyArray<T, N>
where
    T: PackedLayout,
{
    const FOOTPRINT: u64 = N as u64;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Self::lazy(*ExtKeyPtr::next_for::<Self>(ptr))
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        let offset_key = ExtKeyPtr::next_for::<Self>(ptr);
        for (index, entry) in self.cached_entries().iter().enumerate() {
            if let Some(entry) = entry {
                let key = offset_key + (index as u64);
                entry.push_packed_root(&key);
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

impl<T, const N: usize> LazyArray<T, N> {
    /// Returns the offset key for the given index if not out of bounds.
    pub fn key_at(&self, at: Index) -> Option<Key> {
        if at >= self.capacity() {
            return None
        }
        self.key.as_ref().map(|key| key + at as u64)
    }
}

impl<T, const N: usize> LazyArray<T, N>
where
    T: PackedLayout,
{
    /// Loads the entry at the given index.
    ///
    /// Tries to load the entry from cache and falls back to lazily load the
    /// entry from the contract storage.
    fn load_through_cache(&self, at: Index) -> NonNull<StorageEntry<T>> {
        assert!(at < self.capacity(), "index is out of bounds");
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
                let entry = StorageEntry::new(value, EntryState::Preserved);
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
    fn load_through_cache_mut(&mut self, index: Index) -> &mut StorageEntry<T> {
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
        assert!(a < self.capacity(), "a is out of bounds");
        assert!(b < self.capacity(), "b is out of bounds");
        if a == b {
            // Bail out early if both indices are the same.
            return
        }
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

#[cfg(test)]
mod tests {
    use super::{
        super::{
            EntryState,
            StorageEntry,
        },
        Index,
        LazyArray,
    };
    use crate::traits::{
        KeyPtr,
        SpreadLayout,
    };
    use ink_primitives::Key;

    /// Asserts that the cached entries of the given `imap` is equal to the `expected` slice.
    fn assert_cached_entries<const N: usize>(
        larray: &LazyArray<u8, N>,
        expected: &[(Index, StorageEntry<u8>)],
    ) {
        let mut len = 0;
        for (given, expected) in larray
            .cached_entries()
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                match entry {
                    Some(entry) => Some((index as u32, entry)),
                    None => None,
                }
            })
            .zip(expected.iter().map(|(index, entry)| (*index, entry)))
        {
            assert_eq!(given, expected);
            len += 1;
        }
        assert_eq!(len, expected.len());
    }

    #[test]
    fn new_works() {
        let larray = <LazyArray<u8, 4>>::new();
        // Key must be none.
        assert_eq!(larray.key(), None);
        assert_eq!(larray.key_at(0), None);
        assert_eq!(larray.capacity(), 4);
        // Cached elements must be empty.
        assert_cached_entries(&larray, &[]);
        // Same as default:
        let default_larray = <LazyArray<u8, 4>>::default();
        assert_eq!(default_larray.key(), larray.key());
        assert_eq!(default_larray.key_at(0), larray.key_at(0));
        assert_eq!(larray.capacity(), 4);
        assert_cached_entries(&default_larray, &[]);
    }

    #[test]
    fn lazy_works() {
        let key = Key::from([0x42; 32]);
        let larray = <LazyArray<u8, 4>>::lazy(key);
        // Key must be Some.
        assert_eq!(larray.key(), Some(&key));
        assert_eq!(larray.key_at(0), Some(key));
        assert_eq!(larray.key_at(1), Some(key + 1u64));
        assert_eq!(larray.capacity(), 4);
        // Cached elements must be empty.
        assert_cached_entries(&larray, &[]);
    }

    #[test]
    fn get_works() {
        let mut larray = <LazyArray<u8, 4>>::new();
        let nothing_changed = &[
            (0, StorageEntry::new(None, EntryState::Preserved)),
            (1, StorageEntry::new(Some(b'B'), EntryState::Mutated)),
            (2, StorageEntry::new(None, EntryState::Preserved)),
            (3, StorageEntry::new(Some(b'D'), EntryState::Mutated)),
        ];
        // Put some values.
        assert_eq!(larray.put_get(0, None), None);
        assert_eq!(larray.put_get(1, Some(b'B')), None);
        assert_eq!(larray.put_get(2, None), None);
        assert_eq!(larray.put_get(3, Some(b'D')), None);
        assert_cached_entries(&larray, nothing_changed);
        // `get` works:
        assert_eq!(larray.get(0), None);
        assert_eq!(larray.get(1), Some(&b'B'));
        assert_eq!(larray.get(2), None);
        assert_eq!(larray.get(3), Some(&b'D'));
        assert_cached_entries(&larray, nothing_changed);
        // `get_mut` works:
        assert_eq!(larray.get_mut(0), None);
        assert_eq!(larray.get_mut(1), Some(&mut b'B'));
        assert_eq!(larray.get_mut(2), None);
        assert_eq!(larray.get_mut(3), Some(&mut b'D'));
        assert_cached_entries(&larray, nothing_changed);
    }

    #[test]
    #[should_panic(expected = "index is out of bounds")]
    fn get_out_of_bounds_works() {
        let larray = <LazyArray<u8, 4>>::new();
        let _ = larray.get(4);
    }

    #[test]
    fn put_get_works() {
        let mut larray = <LazyArray<u8, 4>>::new();
        // Assert that the array cache is empty at first.
        assert_cached_entries(&larray, &[]);
        // Put none values.
        assert_eq!(larray.put_get(0, None), None);
        assert_eq!(larray.put_get(1, None), None);
        assert_eq!(larray.put_get(3, None), None);
        assert_cached_entries(
            &larray,
            &[
                (0, StorageEntry::new(None, EntryState::Preserved)),
                (1, StorageEntry::new(None, EntryState::Preserved)),
                (3, StorageEntry::new(None, EntryState::Preserved)),
            ],
        );
        // Override with some values.
        assert_eq!(larray.put_get(0, Some(b'A')), None);
        assert_eq!(larray.put_get(1, Some(b'B')), None);
        assert_eq!(larray.put_get(3, None), None);
        assert_cached_entries(
            &larray,
            &[
                (0, StorageEntry::new(Some(b'A'), EntryState::Mutated)),
                (1, StorageEntry::new(Some(b'B'), EntryState::Mutated)),
                (3, StorageEntry::new(None, EntryState::Preserved)),
            ],
        );
        // Override some values with none.
        assert_eq!(larray.put_get(1, None), Some(b'B'));
        assert_eq!(larray.put_get(3, None), None);
        assert_cached_entries(
            &larray,
            &[
                (0, StorageEntry::new(Some(b'A'), EntryState::Mutated)),
                (1, StorageEntry::new(None, EntryState::Mutated)),
                (3, StorageEntry::new(None, EntryState::Preserved)),
            ],
        );
    }

    #[test]
    #[should_panic(expected = "index is out of bounds")]
    fn put_get_out_of_bounds_works() {
        let mut larray = <LazyArray<u8, 4>>::new();
        let _ = larray.put_get(4, Some(b'A'));
    }

    #[test]
    fn put_works() {
        let mut larray = <LazyArray<u8, 4>>::new();
        // Put some values.
        larray.put(0, None);
        larray.put(1, Some(b'B'));
        larray.put(3, None);
        // The main difference between `put` and `put_get` is that `put` never
        // loads from storage which also has one drawback: Putting a `None`
        // value always ends-up in `Mutated` state for the entry even if the
        // entry is already `None`.
        assert_cached_entries(
            &larray,
            &[
                (0, StorageEntry::new(None, EntryState::Mutated)),
                (1, StorageEntry::new(Some(b'B'), EntryState::Mutated)),
                (3, StorageEntry::new(None, EntryState::Mutated)),
            ],
        );
        // Overwrite entries:
        larray.put(0, Some(b'A'));
        larray.put(1, None);
        larray.put(2, Some(b'C'));
        larray.put(3, None);
        assert_cached_entries(
            &larray,
            &[
                (0, StorageEntry::new(Some(b'A'), EntryState::Mutated)),
                (1, StorageEntry::new(None, EntryState::Mutated)),
                (2, StorageEntry::new(Some(b'C'), EntryState::Mutated)),
                (3, StorageEntry::new(None, EntryState::Mutated)),
            ],
        );
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 4 but the index is 4")]
    fn put_out_of_bounds_works() {
        let mut larray = <LazyArray<u8, 4>>::new();
        larray.put(4, Some(b'A'));
    }

    #[test]
    fn swap_works() {
        let mut larray = <LazyArray<u8, 4>>::new();
        let nothing_changed = &[
            (0, StorageEntry::new(Some(b'A'), EntryState::Mutated)),
            (1, StorageEntry::new(Some(b'B'), EntryState::Mutated)),
            (2, StorageEntry::new(None, EntryState::Preserved)),
            (3, StorageEntry::new(None, EntryState::Preserved)),
        ];
        // Put some values.
        assert_eq!(larray.put_get(0, Some(b'A')), None);
        assert_eq!(larray.put_get(1, Some(b'B')), None);
        assert_eq!(larray.put_get(2, None), None);
        assert_eq!(larray.put_get(3, None), None);
        assert_cached_entries(&larray, nothing_changed);
        // Swap same indices: Check that nothing has changed.
        for i in 0..4 {
            larray.swap(i, i);
        }
        assert_cached_entries(&larray, nothing_changed);
        // Swap `None` values: Check that nothing has changed.
        larray.swap(2, 3);
        larray.swap(3, 2);
        assert_cached_entries(&larray, nothing_changed);
        // Swap `Some` and `None`:
        larray.swap(0, 2);
        assert_cached_entries(
            &larray,
            &[
                (0, StorageEntry::new(None, EntryState::Mutated)),
                (1, StorageEntry::new(Some(b'B'), EntryState::Mutated)),
                (2, StorageEntry::new(Some(b'A'), EntryState::Mutated)),
                (3, StorageEntry::new(None, EntryState::Preserved)),
            ],
        );
        // Swap `Some` and `Some`:
        larray.swap(1, 2);
        assert_cached_entries(
            &larray,
            &[
                (0, StorageEntry::new(None, EntryState::Mutated)),
                (1, StorageEntry::new(Some(b'A'), EntryState::Mutated)),
                (2, StorageEntry::new(Some(b'B'), EntryState::Mutated)),
                (3, StorageEntry::new(None, EntryState::Preserved)),
            ],
        );
    }

    #[test]
    #[should_panic(expected = "b is out of bounds")]
    fn swap_rhs_out_of_bounds() {
        let mut larray = <LazyArray<u8, 4>>::new();
        larray.swap(0, 4);
    }

    #[test]
    #[should_panic(expected = "a is out of bounds")]
    fn swap_both_out_of_bounds() {
        let mut larray = <LazyArray<u8, 4>>::new();
        larray.swap(4, 4);
    }

    #[test]
    fn spread_layout_works() -> ink_env::Result<()> {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut larray = <LazyArray<u8, 4>>::new();
            let nothing_changed = &[
                (0, StorageEntry::new(Some(b'A'), EntryState::Mutated)),
                (1, StorageEntry::new(Some(b'B'), EntryState::Mutated)),
                (2, StorageEntry::new(None, EntryState::Preserved)),
                (3, StorageEntry::new(None, EntryState::Preserved)),
            ];
            // Put some values.
            assert_eq!(larray.put_get(0, Some(b'A')), None);
            assert_eq!(larray.put_get(1, Some(b'B')), None);
            assert_eq!(larray.put_get(2, None), None);
            assert_eq!(larray.put_get(3, None), None);
            assert_cached_entries(&larray, nothing_changed);
            // Push the lazy index map onto the contract storage and then load
            // another instance of it from the contract stoarge.
            // Then: Compare both instances to be equal.
            let root_key = Key::from([0x42; 32]);
            SpreadLayout::push_spread(&larray, &mut KeyPtr::from(root_key));
            let larray2 = <LazyArray<u8, 4> as SpreadLayout>::pull_spread(
                &mut KeyPtr::from(root_key),
            );
            assert_cached_entries(&larray2, &[]);
            assert_eq!(larray2.get(0), Some(&b'A'));
            assert_eq!(larray2.get(1), Some(&b'B'));
            assert_eq!(larray2.get(2), None);
            assert_eq!(larray2.get(3), None);
            assert_cached_entries(
                &larray2,
                &[
                    (0, StorageEntry::new(Some(b'A'), EntryState::Preserved)),
                    (1, StorageEntry::new(Some(b'B'), EntryState::Preserved)),
                    (2, StorageEntry::new(None, EntryState::Preserved)),
                    (3, StorageEntry::new(None, EntryState::Preserved)),
                ],
            );
            // Clear the first lazy index map instance and reload another instance
            // to check whether the associated storage has actually been freed
            // again:
            SpreadLayout::clear_spread(&larray2, &mut KeyPtr::from(root_key));
            // The above `clear_spread` call is a no-op since lazy index map is
            // generally not aware of its associated elements. So we have to
            // manually clear them from the contract storage which is what the
            // high-level data structures like `storage::Vec` would command:
            larray2.clear_packed_at(0);
            larray2.clear_packed_at(1);
            larray2.clear_packed_at(2); // Not really needed here.
            larray2.clear_packed_at(3); // Not really needed here.
            let larray3 = <LazyArray<u8, 4> as SpreadLayout>::pull_spread(
                &mut KeyPtr::from(root_key),
            );
            assert_cached_entries(&larray3, &[]);
            assert_eq!(larray3.get(0), None);
            assert_eq!(larray3.get(1), None);
            assert_eq!(larray3.get(2), None);
            assert_eq!(larray3.get(3), None);
            assert_cached_entries(
                &larray3,
                &[
                    (0, StorageEntry::new(None, EntryState::Preserved)),
                    (1, StorageEntry::new(None, EntryState::Preserved)),
                    (2, StorageEntry::new(None, EntryState::Preserved)),
                    (3, StorageEntry::new(None, EntryState::Preserved)),
                ],
            );
            Ok(())
        })
    }
}
