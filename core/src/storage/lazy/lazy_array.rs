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
use crate::storage::{
    KeyPtr,
    PullForward,
    PushForward,
    SaturatingStorage,
    StorageFootprint,
};
use core::{
    cell::UnsafeCell,
    mem,
    ops::Mul,
};
use ink_primitives::Key;
use typenum::{
    Integer,
    Prod,
    P32,
};

/// The index type used in the lazy storage chunk.
pub type Index = u32;

/// The number of cached elements a lazy array holds.
const SIZE: usize = 32;

/// A lazy storage array that spans over 32 storage cells.
///
/// # Note
///
/// Computes operations on the underlying 32 storage cells in a lazy fashion.
/// Due to the size constraints the `LazyArray` is generally more efficient
/// than the [`super::LazyMap`] for most use cases with limited elements.
///
/// This is mainly used as low-level storage primitives by other high-level
/// storage primitives in order to manage the contract storage for a whole
/// chunk of storage cells.
#[derive(Debug)]
pub struct LazyArray<T> {
    /// The offset key for the 32 cells.
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
    cached_entries: UnsafeCell<EntryArray<T>>,
}

/// The underlying array cache for the [`LazyArray`].
#[derive(Debug)]
pub struct EntryArray<T> {
    /// The cache entries of the entry array.
    entries: [Option<Entry<T>>; SIZE],
}

impl<T> EntryArray<T> {
    pub fn new() -> Self {
        Self {
            entries: Default::default(),
        }
    }
}

impl<T> Default for EntryArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> EntryArray<T> {
    /// Puts the the new value into the indexed slot and
    /// returns the old value if any.
    fn put(&mut self, at: Index, new_value: Option<T>) -> Option<T> {
        mem::replace(
            &mut self.entries[at as usize],
            Some(Entry::new(new_value, EntryState::Mutated)),
        )
        .map(Entry::into_value)
        .flatten()
    }

    /// Returns a shared reference to the value at the given index if any.
    fn get(&self, at: Index) -> Option<&T> {
        self.entries[at as usize]
            .as_ref()
            .map(Entry::value)
            .flatten()
    }

    /// Returns a shared reference to the value at the given index if any.
    fn get_mut(&mut self, at: Index) -> Option<&mut T> {
        self.entries[at as usize]
            .as_mut()
            .map(Entry::value_mut)
            .flatten()
    }
}

impl<T> LazyArray<T> {
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
    fn cached_entries(&self) -> &EntryArray<T> {
        unsafe { &*self.cached_entries.get() }
    }

    /// Returns an exclusive reference to the underlying cached entries.
    ///
    /// # Safety
    ///
    /// This operation is safe since it returns an exclusive reference from
    /// a `&mut self` which is viable in safe Rust.
    fn cached_entries_mut(&mut self) -> &mut EntryArray<T> {
        unsafe { &mut *self.cached_entries.get() }
    }

    /// Puts a new value into the given indexed slot.
    ///
    /// # Note
    ///
    /// Use [`Self::put_get`]`(None)` to remove an element.
    pub fn put(&mut self, at: Index, new_value: Option<T>) {
        self.cached_entries_mut().put(at, new_value);
    }
}

impl<T> StorageFootprint for LazyArray<T>
where
    T: StorageFootprint,
    <T as StorageFootprint>::Value: Mul<P32>,
{
    type Value = Prod<<T as StorageFootprint>::Value, P32>;
}

impl<T> PushForward for LazyArray<T>
where
    Self: StorageFootprint,
    <Self as StorageFootprint>::Value: Integer,
    T: StorageFootprint + SaturatingStorage + PushForward,
    <T as StorageFootprint>::Value: Integer,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        let offset_key = ptr.next_for2::<Self>();
        for (index, entry) in self.cached_entries().entries.iter().enumerate() {
            if let Some(entry) = entry {
                if !entry.is_mutated() {
                    continue
                }
                let footprint =
                    <<T as StorageFootprint>::Value as Integer>::to_i64() as u64;
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

impl<T> LazyArray<T>
where
    T: StorageFootprint,
    <T as StorageFootprint>::Value: Integer,
{
    /// Returns the offset key for the given index if not out of bounds.
    pub fn key_at(&self, at: Index) -> Option<Key> {
        if at as usize >= SIZE {
            return None
        }
        self.key.map(|key| {
            key + ((at as u64)
                * <<T as StorageFootprint>::Value as Integer>::to_i64() as u64)
        })
    }
}

impl<T> LazyArray<T>
where
    T: StorageFootprint + PullForward,
{
    /// Returns a shared reference to the element at the given index if any.
    ///
    /// # Note
    ///
    /// This operation eventually loads from contract storage.
    pub fn get(&self, at: Index) -> Option<&T> {
        todo!()
    }

    /// Returns an exclusive reference to the element at the given index if any.
    ///
    /// # Note
    ///
    /// This operation eventually loads from contract storage.
    pub fn get_mut(&mut self, at: Index) -> Option<&mut T> {
        todo!()
    }

    /// Removes the element at the given index and returns it if any.
    ///
    /// # Note
    ///
    /// This operation eventually loads from contract storage.
    pub fn take(&mut self, at: Index) -> Option<T> {
        todo!()
    }

    /// Puts the new value into the indexed slot and returns the old value if any.
    ///
    /// # Note
    ///
    /// - This operation eventually loads from contract storage.
    /// - Prefer [`Self::put`] if you are not interested in the old value.
    /// - Use [`Self::put_get`]`(None)` to remove an element.
    pub fn put_get(&mut self, at: Index, new_value: Option<T>) -> Option<T> {
        todo!()
    }

    /// Swaps the values at indices x and y.
    ///
    /// # Note
    ///
    /// This operation eventually loads from contract storage.
    pub fn swap(&mut self, a: Index, b: Index) {
        todo!()
    }
}
