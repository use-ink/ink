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
    KeyPtr,
    PullForward,
    PushForward,
    StorageSize,
};
use core::{
    cell::{
        Cell,
        UnsafeCell,
    },
    cmp::{
        Eq,
        Ord,
    },
};
use ink_prelude::{
    boxed::Box,
    collections::BTreeMap,
};
use ink_primitives::Key;

/// The index type used in the lazy storage chunk.
pub type Index = u32;

/// Types implementing this trait can use the `LazyMap` in order to
/// convert themselves into an actual storage Key.
///
/// # Note
///
/// By default implemented by `u32` as index for [`ink_core::storage::LazyChunk`]
/// and by `Key` itself as identify function in order to eventually support
/// Solidity-like storage mappings.
pub trait KeyMapping<Value> {
    /// Converts `self` into a storage key using the lazy map parameter.
    fn to_storage_key(&self, offset: &Key) -> Key;
}

impl<Value> KeyMapping<Value> for Index
where
    Value: StorageSize,
{
    fn to_storage_key(&self, offset: &Key) -> Key {
        *offset + (*self as u64 * <Value as StorageSize>::SIZE)
    }
}

impl<Value> KeyMapping<Value> for Key {
    fn to_storage_key(&self, _offset: &Key) -> Key {
        // TODO: Actually implement this correctly similar to how Solidity
        //       handles these cases.
        *self
    }
}

/// A chunk of contiguously stored storage entities indexed by integers.
///
/// # Note
///
/// - Loads each values within the chunk lazily.
/// - This is a low-level storage primitive used by some high-level
///   storage primitives in order to manage the contract storage for a whole
///   chunk of storage cells.
pub type LazyChunk<T> = LazyMap<Index, T>;

/// A Solidity-like mapping of storage entities indexed by key hashes.
///
/// # Note
///
/// - Loads each values within the chunk lazily.
/// - This is a low-level storage primitive used by some high-level
///   storage primitives in order to manage the contract storage similar
///   as to how Solidity mappings distribute their storage entries.
pub type LazyMapping<T> = LazyMap<Key, T>;

/// A lazy storage chunk that spans over a whole chunk of storage cells.
///
/// # Note
///
/// This is mainly used as low-level storage primitives by other high-level
/// storage primitives in order to manage the contract storage for a whole
/// chunk of storage cells.
///
/// A chunk of storage cells is a contiguous range of 2^32 storage cells.
#[derive(Debug)]
pub struct LazyMap<K, V> {
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
    cached_entries: UnsafeCell<EntryMap<K, V>>,
}

impl<K, V> Default for LazyMap<K, V>
where
    K: Ord,
{
    fn default() -> Self {
        Self {
            key: None,
            cached_entries: UnsafeCell::new(EntryMap::default()),
        }
    }
}

/// The map for the contract storage entries.
///
/// # Note
///
/// We keep the whole entry in a `Box<T>` in order to prevent pointer
/// invalidation upon updating the cache through `&self` methods as in
/// [`LazyMap::get`].
pub type EntryMap<K, V> = BTreeMap<K, Box<Entry<V>>>;

/// An entry within the lazy chunk
#[derive(Debug)]
pub struct Entry<T> {
    /// The current value of the entry.
    value: Option<T>,
    /// This is `true` if the `value` has been mutated and is potentially
    /// out-of-sync with the contract storage.
    mutated: Cell<bool>,
}

impl<T> PushForward for Entry<T>
where
    T: PushForward + StorageSize,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        // Reset the mutated entry flag because we just synced.
        self.mutated.set(false);
        // Since `self.value` is of type `Option` this will eventually
        // clear the underlying storage entry if `self.value` is `None`.
        self.value.push_forward(ptr);
    }
}

impl<T> Entry<T> {
    /// Returns `true` if the cached value of the entry has potentially been mutated.
    pub fn mutated(&self) -> bool {
        self.mutated.get()
    }

    /// Returns a shared reference to the value of the entry.
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Returns an exclusive reference to the value of the entry.
    ///
    /// # Note
    ///
    /// This changes the `mutate` state of the entry if the entry was occupied
    /// since the caller could potentially change the returned value.
    pub fn value_mut(&mut self) -> Option<&mut T> {
        self.mutated.set(self.value.is_some());
        self.value.as_mut()
    }

    /// Takes the value from the entry and returns it.
    ///
    /// # Note
    ///
    /// This changes the `mutate` state of the entry if the entry was occupied.
    pub fn take_value(&mut self) -> Option<T> {
        self.mutated.set(self.value.is_some());
        self.value.take()
    }

    /// Puts the new value into the entry and returns the old value.
    ///
    /// # Note
    ///
    /// This changes the `mutate` state of the entry to `true` as long as at
    /// least one of `old_value` and `new_value` is `Some`.
    pub fn put(&mut self, new_value: Option<T>) -> Option<T> {
        match new_value {
            Some(new_value) => {
                self.mutated.set(true);
                self.value.replace(new_value)
            }
            None => self.take_value(),
        }
    }
}

impl<K, V> LazyMap<K, V>
where
    K: KeyMapping<V> + Ord,
{
    /// Returns the storage key associated with the given index.
    pub fn key_at<Q>(&self, at: &Q) -> Option<Key>
    where
        Q: core::borrow::Borrow<K>,
        K: Ord,
    {
        self.key()
            .map(|key| {
                <K as KeyMapping<V>>::to_storage_key(at.borrow(), &key)
            })
    }
}

impl<K, V> LazyMap<K, V>
where
    K: Ord,
{
    /// Creates a new empty lazy chunk that cannot be mutated.
    pub fn new() -> Self {
        Self {
            key: None,
            cached_entries: UnsafeCell::new(EntryMap::new()),
        }
    }

    /// Returns the offset key of the lazy map if any.
    pub fn key(&self) -> Option<&Key> {
        self.key.as_ref()
    }

    /// Returns a shared reference to the underlying entries.
    fn entries(&self) -> &EntryMap<K, V> {
        // SAFETY: It is safe to return a `&` reference from a `&self` receiver.
        unsafe { &*self.cached_entries.get() }
    }

    /// Returns an exclusive reference to the underlying entries.
    fn entries_mut(&mut self) -> &mut EntryMap<K, V> {
        // SAFETY: It is safe to return a `&mut` reference from a `&mut self` receiver.
        unsafe { &mut *self.cached_entries.get() }
    }

    /// Puts the new value at the given index.
    ///
    /// # Note
    ///
    /// - Use [`LazyMap::put`]`(None)` in order to remove an element.
    /// - Prefer this method over [`LazyMap::put_get`] if you are not interested
    ///   in the old value of the same cell index.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the decoding of the old element at the given index failed.
    pub fn put(&mut self, index: K, new_value: Option<V>) {
        self.entries_mut().insert(
            index,
            Box::new(Entry {
                value: new_value,
                mutated: Cell::new(true),
            }),
        );
    }
}

impl<T> StorageSize for LazyChunk<T>
where
    T: StorageSize,
{
    /// A lazy chunk is contiguous and its size can be determined by the
    /// total number of elements it could theoretically hold.
    const SIZE: u64 = <T as StorageSize>::SIZE * (core::u32::MAX as u64);
}

impl<T> StorageSize for LazyMapping<T>
where
    T: StorageSize,
{
    /// A lazy mapping is similar to a Solidity mapping that distributes its
    /// stored entities across the entire contract storage so its inplace size
    /// is actually just 1.
    const SIZE: u64 = 1;
}

impl<K, V> PullForward for LazyMap<K, V>
where
    K: Ord,
    Self: StorageSize,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        Self {
            key: Some(ptr.next_for::<Self>()),
            cached_entries: UnsafeCell::new(BTreeMap::new()),
        }
    }
}

impl<K, V> PushForward for LazyMap<K, V>
where
    Self: StorageSize,
    K: KeyMapping<V> + Ord,
    V: PushForward + StorageSize,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        let key = ptr.next_for::<Self>();
        assert_eq!(self.key, Some(key));
        for (index, entry) in self.entries().iter().filter(|(_, entry)| entry.mutated()) {
            let offset: Key = <K as KeyMapping<V>>::to_storage_key(index, &key);
            let mut ptr = KeyPtr::from(offset);
            PushForward::push_forward(&**entry, &mut ptr);
        }
    }
}

impl<K, V> LazyMap<K, V>
where
    K: KeyMapping<V> + Ord + Eq,
    V: StorageSize + PullForward,
{
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
    unsafe fn lazily_load(&self, index: K) -> *mut Entry<V> {
        let key = match self.key {
            Some(key) => key,
            None => panic!("cannot load lazily in this state"),
        };
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
        let off_key = <K as KeyMapping<V>>::to_storage_key(&index, &key);
        match cached_entries.entry(index) {
            BTreeMapEntry::Occupied(occupied) => &mut **occupied.into_mut(),
            BTreeMapEntry::Vacant(vacant) => {
                let value =
                    <Option<V> as PullForward>::pull_forward(&mut KeyPtr::from(off_key));
                let mutated = Cell::new(false);
                &mut **vacant.insert(Box::new(Entry { value, mutated }))
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
    fn lazily_load_mut(&mut self, index: K) -> &mut Entry<V> {
        // SAFETY:
        // - Returning a `&mut Entry<T>` is safe because entities inside the
        //   cache are stored within a `Box` to not invalidate references into
        //   them upon operating on the outer cache.
        unsafe { &mut *self.lazily_load(index) }
    }

    /// Returns a shared reference to the element at the given index if any.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the decoding of the element at the given index failed.
    pub fn get(&self, index: K) -> Option<&V> {
        // SAFETY: Dereferencing the `*mut T` pointer into a `&T` is safe
        //         since this method's receiver is `&self` so we do not
        //         leak non-shared references to the outside.
        let entry: &Entry<V> = unsafe { &*self.lazily_load(index) };
        entry.value()
    }

    /// Returns an exclusive reference to the element at the given index if any.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the decoding of the element at the given index failed.
    pub fn get_mut(&mut self, index: K) -> Option<&mut V> {
        self.lazily_load_mut(index).value_mut()
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
    pub fn take(&mut self, index: K) -> Option<V> {
        self.lazily_load_mut(index).take_value()
    }
}

impl<K, V> LazyMap<K, V>
where
    K: KeyMapping<V> + Ord + Eq,
    V: StorageSize + PullForward,
{
    /// Puts the new value at the given index and returns the old value if any.
    ///
    /// # Note
    ///
    /// - Use [`LazyMap::put_get`]`(None)` in order to remove an element
    ///   and retrieve the old element back.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the decoding of the old element at the given index failed.
    pub fn put_get(&mut self, index: K, new_value: Option<V>) -> Option<V> {
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
    pub fn swap(&mut self, x: K, y: K) {
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
            unsafe { (&mut *self.lazily_load(x), &mut *self.lazily_load(y)) };
        if loaded_x.value.is_none() && loaded_y.value.is_none() {
            // Bail out since nothing has to be swapped if both values are `None`.
            return
        }
        // Set the `mutate` flag since at this point at least one of the loaded
        // values is guaranteed to be `Some`.
        loaded_x.mutated.set(true);
        loaded_y.mutated.set(true);
        core::mem::swap(&mut loaded_x.value, &mut loaded_y.value);
    }
}
