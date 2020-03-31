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
use crate::{
    hash::{
        hasher::Hasher,
        HashBuilder,
    },
    storage2::{
        KeyPtr,
        PullAt,
        PullForward,
        PushAt,
        PushForward,
        StorageFootprint,
    },
};
use core::{
    cell::{
        RefCell,
        UnsafeCell,
    },
    cmp::{
        Eq,
        Ord,
    },
    ptr::NonNull,
};
use ink_prelude::{
    boxed::Box,
    collections::BTreeMap,
    vec::Vec,
};
use ink_primitives::Key;

/// The map for the contract storage entries.
///
/// # Note
///
/// We keep the whole entry in a `Box<T>` in order to prevent pointer
/// invalidation upon updating the cache through `&self` methods as in
/// [`LazyMap::get`].
pub type EntryMap<K, V> = BTreeMap<K, Box<Entry<V>>>;

/// A lazy storage mapping that stores entries under their SCALE encoded key hashes.
///
/// # Note
///
/// This is mainly used as low-level storage primitives by other high-level
/// storage primitives in order to manage the contract storage for a whole
/// mapping of storage cells.
///
/// This storage data structure might store its entires anywhere in the contract
/// storage. It is the users responsibility to keep track of the entries if it
/// is necessary to do so.
pub struct LazyHashMap<K, V, H> {
    /// The offset key for the storage mapping.
    ///
    /// This offsets the mapping for the entries stored in the contract storage
    /// so that all lazy hash map instances store equal entries at different
    /// locations of the contract storage and avoid collissions.
    key: Option<Key>,
    /// The currently cached entries of the lazy storage mapping.
    ///
    /// This normally only represents a subset of the total set of elements.
    /// An entry is cached as soon as it is loaded or written.
    cached_entries: UnsafeCell<EntryMap<K, V>>,
    /// The used hash builder.
    hash_builder: RefCell<HashBuilder<H, Vec<u8>>>,
}

impl<K, V, H> StorageFootprint for LazyHashMap<K, V, H> {
    /// Actually the `LazyHashMap` requires to store no state on the contract storage.
    /// However, giving it a storage footprint of 1 avoids problems with having multiple
    /// consecutive lazy hash maps in the same type.
    type Value = typenum::U1;
}

impl<K, V, H> PullForward for LazyHashMap<K, V, H>
where
    K: Ord,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        <Self as PullAt>::pull_at(ptr.next_for::<Self>())
    }
}

impl<K, V, H> PullAt for LazyHashMap<K, V, H>
where
    K: Ord,
{
    fn pull_at(at: Key) -> Self {
        Self {
            key: Some(at),
            cached_entries: UnsafeCell::new(EntryMap::new()),
            hash_builder: RefCell::new(HashBuilder::from(Vec::new())),
        }
    }
}

impl<K, V, H, O> PushForward for LazyHashMap<K, V, H>
where
    K: Ord + scale::Encode,
    V: StorageFootprint + PushForward,
    H: Hasher<Output = O>,
    O: Default,
    Key: From<O>,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        <Self as PushAt>::push_at(self, ptr.next_for::<Self>())
    }
}

impl<K, V, H, O> PushAt for LazyHashMap<K, V, H>
where
    K: Ord + scale::Encode,
    V: StorageFootprint + PushForward,
    H: Hasher<Output = O>,
    O: Default,
    Key: From<O>,
{
    fn push_at(&self, at: Key) {
        <Option<Key> as PushAt>::push_at(&Some(at), at);
        for (key, entry) in self
            .entries()
            .iter()
            .filter(|(_, entry)| entry.is_mutated())
        {
            let offset_key = self.to_offset_key(&at, key);
            PushForward::push_forward(&**entry, &mut KeyPtr::from(offset_key));
        }
    }
}

impl<K, V, H> LazyHashMap<K, V, H>
where
    K: Ord,
{
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
            hash_builder: RefCell::new(HashBuilder::from(Vec::new())),
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

    /// Puts the new value under the given key.
    ///
    /// # Note
    ///
    /// - Use [`LazyHashMap::put`]`(None)` in order to remove an element.
    /// - Prefer this method over [`LazyHashMap::put_get`] if you are not interested
    ///   in the old value of the same cell index.
    ///
    /// # Panics
    ///
    /// - If the lazy hash map is in an invalid state that forbids interaction
    ///   with the underlying contract storage.
    /// - If the decoding of the old element at the given index failed.
    pub fn put(&mut self, key: K, new_value: Option<V>) {
        self.entries_mut()
            .insert(key, Box::new(Entry::new(new_value, EntryState::Mutated)));
    }
}

impl<K, V, H, O> LazyHashMap<K, V, H>
where
    K: Ord + scale::Encode,
    H: Hasher<Output = O>,
    O: Default,
    Key: From<O>,
{
    /// Returns an offset key for the given key pair.
    fn to_offset_key(&self, storage_key: &Key, key: &K) -> Key {
        #[derive(scale::Encode)]
        struct KeyPair<'a, K> {
            storage_key: &'a Key,
            value_key: &'a K,
        }
        let key_pair = KeyPair {
            storage_key,
            value_key: key,
        };
        self.hash_builder
            .borrow_mut()
            .hash_encoded(&key_pair)
            .into()
    }

    /// Returns an offset key for the given key.
    fn key_at(&self, key: &K) -> Option<Key> {
        self.key
            .map(|storage_key| self.to_offset_key(&storage_key, key))
    }
}

impl<K, V, H, O> LazyHashMap<K, V, H>
where
    K: Ord + Eq + Clone + scale::Encode,
    V: StorageFootprint + PullForward,
    H: Hasher<Output = O>,
    O: Default,
    Key: From<O>,
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
    unsafe fn lazily_load(&self, key: K) -> NonNull<Entry<V>> {
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
        // We have to clone the key here because we do not have access to the unsafe
        // raw entry API for Rust hash maps, yet since it is unstable. We can remove
        // the contraints on `K: Clone` once we have access to this API.
        // Read more about the issue here: https://github.com/rust-lang/rust/issues/56167
        match cached_entries.entry(key.clone()) {
            BTreeMapEntry::Occupied(occupied) => {
                NonNull::from(&mut **occupied.into_mut())
            }
            BTreeMapEntry::Vacant(vacant) => {
                let offset_key = self
                    .key_at(&key)
                    .expect("cannot load lazily in the current state");
                let value = <Option<V> as PullForward>::pull_forward(&mut KeyPtr::from(
                    offset_key,
                ));
                NonNull::from(
                    &mut **vacant
                        .insert(Box::new(Entry::new(value, EntryState::Preserved))),
                )
            }
        }
    }

    /// Lazily loads the value associated with the given key.
    ///
    /// # Note
    ///
    /// Only loads a value if `key` is set and if the value has not been loaded yet.
    /// Returns a pointer to the freshly loaded or already loaded entry of the value.
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
        unsafe { &mut *self.lazily_load(index).as_ptr() }
    }

    /// Returns a shared reference to the value associated with the given key if any.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the decoding of the element at the given index failed.
    pub fn get(&self, index: K) -> Option<&V> {
        // SAFETY: Dereferencing the `*mut T` pointer into a `&T` is safe
        //         since this method's receiver is `&self` so we do not
        //         leak non-shared references to the outside.
        unsafe { &*self.lazily_load(index).as_ptr() }.value().into()
    }

    /// Returns an exclusive reference to the value associated with the given key if any.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the decoding of the element at the given index failed.
    pub fn get_mut(&mut self, index: K) -> Option<&mut V> {
        self.lazily_load_mut(index).value_mut().into()
    }

    /// Takes and returns the value associated with the given key if any.
    ///
    /// # Note
    ///
    /// This removes the value associated with the given key from the storage.
    ///
    /// # Panics
    ///
    /// - If the lazy chunk is in an invalid state that forbids interaction.
    /// - If the decoding of the element at the given index failed.
    pub fn take(&mut self, key: K) -> Option<V> {
        self.lazily_load_mut(key).take_value()
    }

    /// Puts the new value under the given key and returns the old value if any.
    ///
    /// # Note
    ///
    /// - Use [`LazyHashMap::put_get`]`(None)` in order to remove an element
    ///   and retrieve the old element back.
    ///
    /// # Panics
    ///
    /// - If the lazy hashmap is in an invalid state that forbids interaction.
    /// - If the decoding of the old element at the given index failed.
    pub fn put_get(&mut self, key: K, new_value: Option<V>) -> Option<V> {
        self.lazily_load_mut(key).put(new_value)
    }

    /// Swaps the values at entries with associated keys `x` and `y`.
    ///
    /// This operation tries to be as efficient as possible and reuse allocations.
    ///
    /// # Panics
    ///
    /// - If the lazy hashmap is in an invalid state that forbids interaction.
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
