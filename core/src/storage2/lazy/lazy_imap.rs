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
    ptr::NonNull,
};
use ink_prelude::{
    boxed::Box,
    collections::BTreeMap,
};
use ink_primitives::Key;

/// The index type used in the lazy storage chunk.
pub type Index = u32;

/// A lazy storage chunk that spans over a whole chunk of storage cells.
///
/// # Note
///
/// This is mainly used as low-level storage primitives by other high-level
/// storage primitives in order to manage the contract storage for a whole
/// chunk of storage cells.
///
/// A chunk of storage cells is a contiguous range of 2^32 storage cells.
pub struct LazyIndexMap<V> {
    /// The offset key for the chunk of cells.
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
    cached_entries: UnsafeCell<EntryMap<V>>,
}

struct DebugEntryMap<'a, V>(&'a UnsafeCell<EntryMap<V>>);

impl<'a, V> Debug for DebugEntryMap<'a, V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entries(unsafe { &*self.0.get() }.iter())
            .finish()
    }
}

impl<V> Debug for LazyIndexMap<V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LazyIndexMap")
            .field("key", &self.key)
            .field("cached_entries", &DebugEntryMap(&self.cached_entries))
            .finish()
    }
}

#[test]
fn debug_impl_works() {
    let mut imap = <LazyIndexMap<i32>>::new();
    // Empty imap.
    assert_eq!(
        format!("{:?}", &imap),
        "LazyIndexMap { key: None, cached_entries: {} }",
    );
    // Filled imap.
    imap.put(0, Some(1));
    imap.put(42, Some(2));
    imap.put(999, None);
    assert_eq!(
        format!("{:?}", &imap),
        "LazyIndexMap { \
            key: None, \
            cached_entries: {\
                0: Entry { \
                    value: Some(1), \
                    state: Mutated \
                }, \
                42: Entry { \
                    value: Some(2), \
                    state: Mutated \
                }, \
                999: Entry { \
                    value: None, \
                    state: Mutated \
                }\
            } \
        }",
    );
}

impl<V> Default for LazyIndexMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

/// The map for the contract storage entries.
///
/// # Note
///
/// We keep the whole entry in a `Box<T>` in order to prevent pointer
/// invalidation upon updating the cache through `&self` methods as in
/// [`LazyIndexMap::get`].
pub type EntryMap<V> = BTreeMap<Index, Box<Entry<V>>>;

use super::{
    Entry,
    EntryState,
};

impl<V> LazyIndexMap<V> {
    /// Creates a new empty lazy map.
    ///
    /// # Note
    ///
    /// A lazy map created this way cannot be used to load from the contract storage.
    /// All operations that directly or indirectly load from storage will panic.
    pub fn new() -> Self {
        Self {
            key: None,
            cached_entries: UnsafeCell::new(EntryMap::new()),
        }
    }

    /// Creates a new empty lazy map positioned at the given key.
    ///
    /// # Note
    ///
    /// This constructor is private and should never need to be called from
    /// outside this module. It is used to construct a lazy index map from a
    /// key that is only useful upon a contract call. Use [`LazyIndexMap::new`]
    /// for construction during contract initialization.
    fn lazy(key: Key) -> Self {
        Self {
            key: Some(key),
            cached_entries: UnsafeCell::new(EntryMap::new()),
        }
    }

    /// Returns the offset key of the lazy map if any.
    pub fn key(&self) -> Option<&Key> {
        self.key.as_ref()
    }

    /// Returns a shared reference to the underlying entries.
    fn entries(&self) -> &EntryMap<V> {
        // SAFETY: It is safe to return a `&` reference from a `&self` receiver.
        unsafe { &*self.cached_entries.get() }
    }

    /// Returns an exclusive reference to the underlying entries.
    fn entries_mut(&mut self) -> &mut EntryMap<V> {
        // SAFETY: It is safe to return a `&mut` reference from a `&mut self` receiver.
        unsafe { &mut *self.cached_entries.get() }
    }

    /// Puts the new value at the given index.
    ///
    /// # Note
    ///
    /// - Use [`LazyIndexMap::put`]`(None)` in order to remove an element.
    /// - Prefer this method over [`LazyIndexMap::put_get`] if you are not interested
    ///   in the old value of the same cell index.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the decoding of the old element at the given index failed.
    pub fn put(&mut self, index: Index, new_value: Option<V>) {
        self.entries_mut()
            .insert(index, Box::new(Entry::new(new_value, EntryState::Mutated)));
    }
}

impl<V> SpreadLayout for LazyIndexMap<V>
where
    V: PackedLayout,
{
    const FOOTPRINT: u64 = 1_u64 << 32;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Self::lazy(ptr.next_for::<Self>())
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        let offset_key = ptr.next_for::<Self>();
        for (&index, entry) in self.entries().iter() {
            let root_key = offset_key + index;
            entry.push_packed_root(&root_key);
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

impl<V> LazyIndexMap<V>
where
    V: PackedLayout,
{
    /// Clears the underlying storage of the entry at the given index.
    ///
    /// # Safety
    ///
    /// For performance reasons this does not synchronize the lazy index map's
    /// memory-side cache which invalidates future accesses the cleared entry.
    /// Care should be taken when using this API.
    ///
    /// The general use of this API is to streamline `Drop` implementations of
    /// high-level abstractions that build upon this low-level data strcuture.
    pub fn clear_packed_at(&self, index: Index) {
        let root_key = self.key.expect("cannot clear in lazy state");
        // We need to load the entity before we remove its associated contract storage
        // because it might be a type that propagates clearing to its fields,
        // for example in the case of `T` being a `storage::Box`. However,
        // since in other cases this load is not required but implies a lot of
        // overhead we need to find a way to avoid it in those cases.
        let entity = self.get(index).expect("cannot clear a non existing entity");
        clear_packed_root::<V>(&entity, &root_key);
    }
}

impl<V> LazyIndexMap<V>
where
    V: PackedLayout,
{
    /// Returns an offset key for the given index.
    pub fn key_at(&self, index: Index) -> Option<Key> {
        let key = self.key?;
        let offset_key = key + index as u64;
        Some(offset_key)
    }

    /// Lazily loads the value at the given index.
    ///
    /// # Note
    ///
    /// Only loads a value if `key` is set and if the value has not been loaded yet.
    /// Returns the freshly loaded or already loaded entry of the value.
    ///
    /// # Safety
    ///
    /// This function has a `&self` receiver while returning an `Option<*mut T>`
    /// which is unsafe in isolation. The caller has to determine how to forward
    /// the returned `*mut T`.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the lazy chunk is not in a state that allows lazy loading.
    ///
    /// # Safety
    ///
    /// This is an `unsafe` operation because it has a `&self` receiver but returns
    /// a `*mut Entry<T>` pointer that allows for exclusive access. This is safe
    /// within internal use only and should never be given outside of the lazy
    /// entity for public `&self` methods.
    unsafe fn lazily_load(&self, index: Index) -> NonNull<Entry<V>> {
        // SAFETY: We have put the whole `cached_entries` mapping into an
        //         `UnsafeCell` because of this caching functionality. The
        //         trick here is that due to using `Box<T>` internally
        //         we are able to return references to the cached entries
        //         while maintaining the invariant that mutating the caching
        //         `BTreeMap` will never invalidate those references.
        //         By returning a raw pointer we enforce an `unsafe` block at
        //         the caller site to underline that guarantees are given by the
        //         caller.
        #[allow(unused_unsafe)]
        let cached_entries = unsafe { &mut *self.cached_entries.get() };
        use ink_prelude::collections::btree_map::Entry as BTreeMapEntry;
        match cached_entries.entry(index) {
            BTreeMapEntry::Occupied(occupied) => {
                NonNull::from(&mut **occupied.into_mut())
            }
            BTreeMapEntry::Vacant(vacant) => {
                let offset_key = self
                    .key_at(index)
                    .expect("cannot load lazily in this state");
                let value = pull_packed_root_opt::<V>(&offset_key);
                NonNull::from(
                    &mut **vacant
                        .insert(Box::new(Entry::new(value, EntryState::Preserved))),
                )
            }
        }
    }

    /// Lazily loads the value at the given index.
    ///
    /// # Note
    ///
    /// Only loads a value if `key` is set and if the value has not been loaded yet.
    /// Returns the freshly loaded or already loaded entry of the value.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the lazy chunk is not in a state that allows lazy loading.
    fn lazily_load_mut(&mut self, index: Index) -> &mut Entry<V> {
        // SAFETY:
        // - Returning a `&mut Entry<T>` is safe because entities inside the
        //   cache are stored within a `Box` to not invalidate references into
        //   them upon operating on the outer cache.
        unsafe { &mut *self.lazily_load(index).as_ptr() }
    }

    /// Returns a shared reference to the element at the given index if any.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the decoding of the element at the given index failed.
    pub fn get(&self, index: Index) -> Option<&V> {
        // SAFETY: Dereferencing the `*mut T` pointer into a `&T` is safe
        //         since this method's receiver is `&self` so we do not
        //         leak non-shared references to the outside.
        unsafe { &*self.lazily_load(index).as_ptr() }.value().into()
    }

    /// Returns an exclusive reference to the element at the given index if any.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the decoding of the element at the given index failed.
    pub fn get_mut(&mut self, index: Index) -> Option<&mut V> {
        self.lazily_load_mut(index).value_mut().into()
    }

    /// Takes and returns the element at the given index if any.
    ///
    /// # Note
    ///
    /// This removes the element at the given index from the storage.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the decoding of the element at the given index failed.
    pub fn take(&mut self, index: Index) -> Option<V> {
        self.lazily_load_mut(index).take_value()
    }

    /// Puts the new value at the given index and returns the old value if any.
    ///
    /// # Note
    ///
    /// - Use [`LazyIndexMap::put_get`]`(None)` in order to remove an element
    ///   and retrieve the old element back.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the decoding of the old element at the given index failed.
    pub fn put_get(&mut self, index: Index, new_value: Option<V>) -> Option<V> {
        self.lazily_load_mut(index).put(new_value)
    }

    /// Swaps the values at indices `x` and `y`.
    ///
    /// This operation tries to be as efficient as possible and reuse allocations.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the decoding of one of the elements failed.
    pub fn swap(&mut self, x: Index, y: Index) {
        if x == y {
            // Bail out early if both indices are the same.
            return
        }
        let (loaded_x, loaded_y) =
            // SAFETY: The loaded `x` and `y` entries are distinct from each
            //         other guaranteed by the previous check. Also `lazily_load`
            //         guarantees to return a pointer to a pinned entity
            //         so that the returned references do not conflict with
            //         each other.
            unsafe { (
                &mut *self.lazily_load(x).as_ptr(),
                &mut *self.lazily_load(y).as_ptr(),
            ) };
        if loaded_x.value().is_none() && loaded_y.value().is_none() {
            // Bail out since nothing has to be swapped if both values are `None`.
            return
        }
        // Set the `mutate` flag since at this point at least one of the loaded
        // values is guaranteed to be `Some`.
        loaded_x.set_state(EntryState::Mutated);
        loaded_y.set_state(EntryState::Mutated);
        core::mem::swap(loaded_x.value_mut(), loaded_y.value_mut());
    }
}
