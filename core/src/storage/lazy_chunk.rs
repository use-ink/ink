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
use core::{
    cell::UnsafeCell,
    pin::Pin,
};
use ink_primitives::Key;

use ink_prelude::{
    boxed::Box,
    collections::BTreeMap,
};

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
#[derive(Debug, Default)]
pub struct LazyChunk<T> {
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
    cached_entries: UnsafeCell<EntryMap<T>>,
}

/// The map for the contract storage entries.
pub type EntryMap<T> = BTreeMap<Index, Entry<T>>;

/// An entry within the lazy chunk
#[derive(Debug)]
pub struct Entry<T> {
    /// The current value of the entry.
    ///
    /// We keep the value in a `Pin<Box<T>>` in order to prevent pointer
    /// invalidation upon updating the cache through `&self` methods as in
    /// [`LazyChunk::get`].
    value: Option<Pin<Box<T>>>,
    /// This is `true` if the `value` has been mutated and is potentially
    /// out-of-sync with the contract storage.
    mutated: bool,
}

impl<T> Entry<T> {
    /// Returns a shared reference to the value of the entry.
    pub fn value(&self) -> Option<&T> {
        match &self.value {
            Some(value) => Some(value.as_ref().get_ref()),
            None => None,
        }
    }

    /// Returns an exclusive reference to the value of the entry.
    ///
    /// # Note
    ///
    /// This changes the `mutate` state of the entry if the entry was occupied
    /// since the caller could potentially change the returned value.
    pub fn value_mut(&mut self) -> Option<&mut T>
    where
        T: Unpin,
    {
        match &mut self.value {
            Some(value) => {
                self.mutated = true;
                Some(value.as_mut().get_mut())
            }
            None => None,
        }
    }

    /// Takes the value from the entry and returns it.
    ///
    /// # Note
    ///
    /// This changes the `mutate` state of the entry if the entry was occupied.
    pub fn take_value(&mut self) -> Option<T>
    where
        T: Unpin,
    {
        if self.value.is_some() {
            self.mutated = true;
        }
        self.value.take().map(|pin| *Pin::into_inner(pin))
    }

    /// Puts the new value into the entry and returns the old value.
    ///
    /// # Note
    ///
    /// This changes the `mutate` state of the entry to `true` as long as at
    /// least one of `old_value` and `new_value` is `Some`.
    pub fn put(&mut self, new_value: Option<T>) -> Option<T>
    where
        T: Unpin,
    {
        if self.value.is_some() || new_value.is_some() {
            self.mutated = true;
        }
        // Note: This implementation is a bit more complex than it could be
        //       because we want to re-use the eventually already heap allocated
        //       `Pin<Box<T>>`.
        match &mut self.value {
            old_value @ Some(_) => {
                match new_value {
                    Some(new_value) => {
                        // Re-use the heap allocation.
                        let old_value = old_value
                            .as_mut()
                            .expect("we asserted to have a value")
                            .as_mut()
                            .get_mut();
                        Some(core::mem::replace::<T>(old_value, new_value))
                    }
                    None => {
                        // Throw-away the heap allocation.
                        let old_value =
                            old_value.take().expect("we asserted to have a value");
                        Some(*Pin::into_inner(old_value))
                    }
                }
            }
            old_value @ None => {
                match new_value {
                    Some(new_value) => {
                        // Create a new heap allocation.
                        old_value.replace(Box::pin(new_value));
                        None
                    }
                    None => {
                        // We do nothing.
                        None
                    }
                }
            }
        }
    }
}

impl<T> LazyChunk<T> {
    /// Creates a new empty lazy chunk that cannot be mutated.
    pub fn new() -> Self {
        Self {
            key: None,
            cached_entries: UnsafeCell::new(EntryMap::new()),
        }
    }

    /// Performs the given closure on the mutable cached entries.
    ///
    /// # Note
    ///
    /// Actions on the mutable lazy entries are performed within the closure
    /// to not leak exclusive references to it to the outside. This is important
    /// since the `for_entries` method itself operates only on `&self`.
    fn for_cached_entries<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut EntryMap<T>) -> R,
    {
        // SAFETY: We operate on an exclusive reference on `BTreeMap` within the
        //         given closure while our method receiver is only a shared reference.
        //         However, due to encapsulation of the exclusive reference within
        //         the given closure we cannot leak the exclusive reference outside
        //         of the closure. So the below action is safe in this regard.
        f(unsafe { &mut *self.cached_entries.get() })
    }
}

impl<T> StorageSize for LazyChunk<T> {
    const SIZE: u64 = core::u32::MAX as u64;
}

impl<T> Pull for LazyChunk<T>
where
    Self: StorageSize,
{
    fn pull(key_ptr: &mut KeyPtr) -> Self {
        Self {
            key: Some(key_ptr.next_for::<Self>()),
            cached_entries: UnsafeCell::new(BTreeMap::new()),
        }
    }
}

impl<T> Push for LazyChunk<T>
where
    T: Push + scale::Encode,
{
    fn push(&self, key_ptr: &mut KeyPtr) {
        // Simply increment `key_ptr` for the next storage entities.
        let next_key = key_ptr.next_for::<Self>();
        match self.key {
            None => (),
            Some(key) => {
                assert_eq!(
                    key, next_key,
                    // Panic if we would push to some other place than we'd pull from.
                    // This is just us being overly assertive.
                    "pull and push keys do not match"
                );
                self.for_cached_entries(|entries| {
                    for (&index, entry) in entries.iter_mut().filter(|(_, entry)| entry.mutated) {
                        let offset: Key = key + index;
                        let mut ptr = KeyPtr::from(offset);
                        match &entry.value {
                            Some(value) => {
                                // Forward push to the inner value on the computed key.
                                Push::push(&**value, &mut ptr);
                            }
                            None => {
                                // Clear the storage at the given index.
                                crate::env::clear_contract_storage(offset);
                            }
                        }
                        // Reset the mutated entry flag because we just synced.
                        entry.mutated = false;
                    }
                });
            }
        }
    }
}

impl<T> LazyChunk<T>
where
    T: scale::Decode,
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
    /// If the lazy chunk is not in a state that allows lazy loading.
    fn lazily_load(&self, index: Index) -> *mut Entry<T>
    where
        T: Unpin,
    {
        let key = match self.key {
            Some(key) => key,
            None => panic!("cannot load lazily in this state"),
        };
        // SAFETY: We have put the whole `cached_entries` mapping into an
        //         `UnsafeCell` because of this caching functionality. The
        //         trick here is that due to using `Pin<Box<T>>` internally
        //         we are able to return references to the cached entries
        //         while maintaining the invariant that mutating the caching
        //         `BTreeMap` will never invalidate those references.
        //         By returning a raw pointer we enforce an `unsafe` block at
        //         the caller site to underline that guarantees are given by the
        //         caller.
        let cached_entries = unsafe { &mut *self.cached_entries.get() };
        use ink_prelude::collections::btree_map::Entry as BTreeMapEntry;
        match cached_entries.entry(index) {
            BTreeMapEntry::Occupied(occupied) => occupied.into_mut(),
            BTreeMapEntry::Vacant(vacant) => {
                let loaded_value =
                    match crate::env::get_contract_storage::<T>(key + index) {
                        Some(new_value) => {
                            Some(new_value.expect("could not decode lazily loaded value"))
                        }
                        None => None,
                    };
                let value = loaded_value.map(|value| Box::pin(value));
                vacant.insert(Entry {
                    value,
                    mutated: false,
                })
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
    /// If the lazy chunk is not in a state that allows lazy loading.
    fn lazily_load_mut(&mut self, index: Index) -> &mut Entry<T>
    where
        T: Unpin,
    {
        // SAFETY: Dereferencing the raw-pointer here is safe since we
        //         encapsulated this whole call with a `&mut self` receiver.
        unsafe { &mut *self.lazily_load(index) }
    }

    /// Returns a shared reference to the element at the given index if any.
    ///
    /// # Panics
    ///
    /// If the decoding of the element at the given index failed.
    pub fn get(&self, index: Index) -> Option<&T>
    where
        T: Unpin,
    {
        // SAFETY: Dereferencing the `*mut T` pointer into a `&T` is safe
        //         since this method's receiver is `&self` so we do not
        //         leak non-shared references to the outside.
        let entry: &Entry<T> = unsafe { &*self.lazily_load(index) };
        entry.value()
    }

    /// Returns an exclusive reference to the element at the given index if any.
    ///
    /// # Panics
    ///
    /// If the decoding of the element at the given index failed.
    pub fn get_mut(&mut self, index: Index) -> Option<&mut T>
    where
        T: Unpin,
    {
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
    /// If the decoding of the element at the given index failed.
    pub fn take(&mut self, index: Index) -> Option<T>
    where
        T: Unpin,
    {
        self.lazily_load_mut(index).take_value()
    }
}

impl<T> LazyChunk<T>
where
    T: scale::Encode,
{
    /// Puts the new value at the given index and returns the old value if any.
    ///
    /// # Note
    ///
    /// - Use [`LazyChunk::put`]`(None)` in order to remove an element.
    /// - Prefer this method over [`LazyChunk::put_get`] if you are not interested
    ///   in the old value of the same cell index.
    ///
    /// # Panics
    ///
    /// If the decoding of the old element at the given index failed.
    pub fn put(&mut self, _index: Index, _new_value: Option<T>) {
        todo!()
    }
}

impl<T> LazyChunk<T>
where
    T: Unpin + scale::Codec,
{
    /// Puts the new value at the given index and returns the old value if any.
    ///
    /// # Note
    ///
    /// - Use [`LazyChunk::put_get`]`(None)` in order to remove an element
    ///   and retrieve the old element back.
    ///
    /// # Panics
    ///
    /// If the decoding of the old element at the given index failed.
    pub fn put_get(&mut self, index: Index, new_value: Option<T>) -> Option<T> {
        self.lazily_load_mut(index).put(new_value)
    }
}
