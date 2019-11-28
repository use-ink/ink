// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use core::cell::RefCell;
use ink_prelude::collections::btree_map::{
    BTreeMap,
    Entry,
};

/// A single cache entry.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CacheValue<T> {
    /// If the cache for this entry is dirty.
    dirty: bool,
    /// The value of the cached cell.
    value: Option<T>,
}

impl<T> CacheValue<T> {
    /// Creates a new cache value.
    ///
    /// # Note
    ///
    /// It is marked clean after creation.
    fn new(cell_val: Option<T>) -> Self {
        Self {
            dirty: false,
            value: cell_val,
        }
    }

    /// Returns `true` if the cached value is dirty.
    fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks the cached value as being dirty.
    fn mark_dirty(&mut self) {
        self.dirty = true
    }

    /// Marks the cached value as being clean.
    pub fn mark_clean(&mut self) {
        self.dirty = false
    }

    /// Returns an immutable reference to the cached value.
    pub fn get(&self) -> Option<&T> {
        (&self.value).into()
    }

    /// Returns a mutable reference to the cached value.
    ///
    /// # Note
    ///
    /// Marks the cached value as dirty.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.mark_dirty();
        (&mut self.value).into()
    }

    /// Updates the value of the cached cell.
    ///
    /// Returns an immutable reference to the updated cell value.
    fn update(&mut self, new_val: Option<T>) -> Option<&T> {
        self.value = new_val;
        self.get()
    }

    /// Updates the value of the cached cell.
    ///
    /// Returns a mutable reference to the updated cell value.
    ///
    /// # Note
    ///
    /// Marks the cached value as dirty.
    fn update_mut(&mut self, new_val: Option<T>) -> Option<&mut T> {
        self.value = new_val;
        self.get_mut()
    }

    /// Takes the cell value from the cache leaving the cache value empty.
    ///
    /// # Note
    ///
    /// Marks the cached value as dirty.
    pub fn take(&mut self) -> Option<T> {
        self.put(None)
    }

    /// Replaces the cell value from the cache with the new value.
    ///
    /// # Note
    ///
    /// Marks the cache value as dirty.
    pub fn put(&mut self, new_val: Option<T>) -> Option<T> {
        let cell_val = core::mem::replace(&mut self.value, new_val);
        self.mark_dirty();
        cell_val
    }
}

/// A cache for synchronized cell values.
#[derive(Debug)]
struct Cache<T> {
    /// Cached entries of the cache.
    entries: BTreeMap<u32, CacheValue<T>>,
}

impl<T> Default for Cache<T> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::default(),
        }
    }
}

/// A single cache entry for a copy chunk cell.
type CacheEntry<'a, T> = Entry<'a, u32, CacheValue<T>>;

impl<T> Cache<T> {
    /// Returns an immutable reference to the cached value at position `n` if any.
    fn get(&self, n: u32) -> Option<&CacheValue<T>> {
        self.entries.get(&n)
    }

    /// Returns a mutable reference to the cached value at position `n` if any.
    fn get_mut(&mut self, n: u32) -> Option<&mut CacheValue<T>> {
        self.entries.get_mut(&n)
    }

    /// Returns the cache entry at position `n`.
    fn entry_at(&mut self, n: u32) -> CacheEntry<T> {
        self.entries.entry(n)
    }

    /// Updates the cell value of the cached cell at position `n`.
    pub fn update(&mut self, n: u32, new_val: Option<T>) -> Option<&T> {
        match self.entry_at(n) {
            Entry::Occupied(occupied) => occupied.into_mut().update(new_val),
            Entry::Vacant(vacant) => vacant.insert(CacheValue::new(new_val)).get(),
        }
    }

    /// Updates the cell value of the cached cell at position `n`.
    ///
    /// # Note
    ///
    /// Marks the cached value as dirty.
    pub fn update_mut(&mut self, n: u32, new_val: Option<T>) -> Option<&mut T> {
        match self.entry_at(n) {
            Entry::Occupied(occupied) => occupied.into_mut().update_mut(new_val),
            Entry::Vacant(vacant) => vacant.insert(CacheValue::new(new_val)).get_mut(),
        }
    }

    /// Iterator over all dirty marked cache values.
    pub fn iter_dirty(&mut self) -> impl Iterator<Item = (u32, &mut CacheValue<T>)> {
        self.entries
            .iter_mut()
            .filter(|(_, v)| v.is_dirty())
            .map(|(&k, v)| (k, v))
    }
}

/// A cache guard for synchronized cell values.
///
/// # Note
///
/// Can be used mutable through immutable reference.
#[derive(Debug)]
pub(crate) struct CacheGuard<T> {
    cache: RefCell<Cache<T>>,
}

impl<T> Default for CacheGuard<T> {
    fn default() -> Self {
        Self {
            cache: RefCell::new(Default::default()),
        }
    }
}

impl<T> CacheGuard<T> {
    /// Returns an immutable reference to the internal cache entry.
    ///
    /// Used to returns references from the inside to the outside.
    fn elems(&self) -> &Cache<T> {
        unsafe { &*self.cache.as_ptr() }
    }

    /// Returns an immutable reference to the internal cache entry.
    ///
    /// Used to returns references from the inside to the outside.
    ///
    /// # Devs & Internals
    ///
    /// Note the very critically looking `allow(clippy::mut_from_ref)`.
    /// We might change this in the future and we should be very careful
    /// about its usage!
    #[allow(clippy::mut_from_ref)]
    fn elems_mut(&self) -> &mut Cache<T> {
        unsafe { &mut *self.cache.as_ptr() }
    }

    /// Returns an immutable reference to the cached value at position `n` if any.
    pub fn get(&self, n: u32) -> Option<&CacheValue<T>> {
        self.elems().get(n)
    }

    /// Returns a mutable reference to the cached value at position `n` if any.
    pub fn get_mut(&self, n: u32) -> Option<&mut CacheValue<T>> {
        self.elems_mut().get_mut(n)
    }

    /// Updates the cell value of the cached cell at position `n`.
    ///
    /// Returns an immutable reference to the updated cached value.
    pub fn update(&self, n: u32, new_val: Option<T>) -> Option<&T> {
        self.elems_mut().update(n, new_val)
    }

    /// Updates the cell value of the cached cell at position `n`.
    ///
    /// Returns an mutable reference to the updated cached value.
    ///
    /// # Note
    ///
    /// Marks the cached value as dirty.
    pub fn update_mut(&self, n: u32, new_val: Option<T>) -> Option<&mut T> {
        self.elems_mut().update_mut(n, new_val)
    }

    /// Iterator over all dirty marked cache values.
    pub fn iter_dirty(&self) -> impl Iterator<Item = (u32, &mut CacheValue<T>)> {
        self.elems_mut().iter_dirty()
    }
}
