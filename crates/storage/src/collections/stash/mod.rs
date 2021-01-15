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

//! A storage stash allowing to store indexed elements efficiently.

mod impls;
mod iter;
mod storage;

#[cfg(test)]
mod tests;

use self::iter::Entries;
pub use self::iter::{
    Iter,
    IterMut,
};
use crate::{
    lazy::LazyIndexMap,
    traits::PackedLayout,
    Pack,
};
use ink_primitives::Key;

/// An index into the stash.
type Index = u32;

/// A stash data structure operating on contract storage.
///
/// This allows to store information similar to a vector but in unordered
/// fashion which enables constant time random deletion of elements. This allows
/// for efficient attachment of data to some numeric indices.
#[derive(Debug)]
pub struct Stash<T>
where
    T: PackedLayout,
{
    /// The combined and commonly used header data.
    header: Pack<Header>,
    /// The storage entries of the stash.
    entries: LazyIndexMap<Entry<T>>,
}

/// Stores general commonly required information about the storage stash.
#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
struct Header {
    /// The latest vacant index.
    ///
    /// - If all entries are occupied:
    ///     - Points to the entry at index `self.len`.
    /// - If some entries are vacant:
    ///     - Points to the entry that has been vacated most recently.
    last_vacant: Index,
    /// The number of items stored in the stash.
    ///
    /// # Note
    ///
    /// We cannot simply use the underlying length of the vector
    /// since it would include vacant slots as well.
    len: u32,
    /// The number of entries currently managed by the stash.
    len_entries: u32,
}

/// A vacant entry with previous and next vacant indices.
#[derive(Debug, Copy, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct VacantEntry {
    /// The next vacant index.
    next: Index,
    /// The previous vacant index.
    prev: Index,
}

/// An entry within the stash.
///
/// The vacant entries within a storage stash form a doubly linked list of
/// vacant entries that is used to quickly re-use their vacant storage.
#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Entry<T> {
    /// A vacant entry that holds the index to the next and previous vacant entry.
    Vacant(VacantEntry),
    /// An occupied entry that hold the value.
    Occupied(T),
}

impl<T> Entry<T> {
    /// Returns `true` if the entry is occupied.
    pub fn is_occupied(&self) -> bool {
        if let Entry::Occupied(_) = self {
            return true
        }
        false
    }

    /// Returns `true` if the entry is vacant.
    pub fn is_vacant(&self) -> bool {
        !self.is_occupied()
    }

    /// Returns the vacant entry if the entry is vacant, otherwise returns `None`.
    fn try_to_vacant(&self) -> Option<VacantEntry> {
        match self {
            Entry::Occupied(_) => None,
            Entry::Vacant(vacant_entry) => Some(*vacant_entry),
        }
    }

    /// Returns the vacant entry if the entry is vacant, otherwise returns `None`.
    fn try_to_vacant_mut(&mut self) -> Option<&mut VacantEntry> {
        match self {
            Entry::Occupied(_) => None,
            Entry::Vacant(vacant_entry) => Some(vacant_entry),
        }
    }
}

impl<T> Stash<T>
where
    T: PackedLayout,
{
    /// Creates a new empty stash.
    pub fn new() -> Self {
        Self {
            header: Pack::new(Header {
                last_vacant: 0,
                len: 0,
                len_entries: 0,
            }),
            entries: LazyIndexMap::new(),
        }
    }

    /// Returns the number of elements stored in the stash.
    pub fn len(&self) -> u32 {
        self.header.len
    }

    /// Returns `true` if the stash contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of entries the stash can hold without
    /// allocating another storage cell.
    ///
    /// # Note
    ///
    /// This is the total number of occupied and vacant entries of the stash.
    pub fn capacity(&self) -> u32 {
        self.len_entries()
    }

    /// Returns the number of entries currently managed by the storage stash.
    fn len_entries(&self) -> u32 {
        self.header.len_entries
    }

    /// Returns the underlying key to the cells.
    ///
    /// # Note
    ///
    /// This is a low-level utility getter and should
    /// normally not be required by users.
    pub fn entries_key(&self) -> Option<&Key> {
        self.entries.key()
    }

    /// Returns an iterator yielding shared references to all elements of the stash.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over big storage stashes.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }

    /// Returns an iterator yielding exclusive references to all elements of the stash.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over big storage stashes.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut::new(self)
    }

    /// Returns an iterator yielding shared references to all entries of the stash.
    pub fn entries(&self) -> Entries<T> {
        Entries::new(self)
    }

    /// Returns `true` if the storage stash has vacant entries.
    fn has_vacant_entries(&self) -> bool {
        self.header.len != self.header.len_entries
    }

    /// Returns the index of the last vacant entry if any.
    fn last_vacant_index(&self) -> Option<Index> {
        if self.has_vacant_entries() {
            Some(self.header.last_vacant)
        } else {
            None
        }
    }
}

impl<T> Stash<T>
where
    T: PackedLayout,
{
    /// Returns a shared reference to the element at the given index.
    pub fn get(&self, at: Index) -> Option<&T> {
        if at >= self.len_entries() {
            // Bail out early if the index is out of bounds.
            return None
        }
        self.entries.get(at).and_then(|entry| {
            match entry {
                Entry::Occupied(val) => Some(val),
                Entry::Vacant { .. } => None,
            }
        })
    }

    /// Returns an exclusive reference to the element at the given index.
    pub fn get_mut(&mut self, at: Index) -> Option<&mut T> {
        if at >= self.len_entries() {
            // Bail out early if the index is out of bounds.
            return None
        }
        self.entries.get_mut(at).and_then(|entry| {
            match entry {
                Entry::Occupied(val) => Some(val),
                Entry::Vacant { .. } => None,
            }
        })
    }
}

impl<T> Stash<T>
where
    T: PackedLayout,
{
    /// Clears the underlying storage cells of the storage vector.
    ///
    /// # Note
    ///
    /// This completely invalidates the storage vector's invariants about
    /// the contents of its associated storage region.
    ///
    /// This API is used for the `Drop` implementation of [`Vec`] as well as
    /// for the [`SpreadLayout::clear_spread`] trait implementation.
    fn clear_cells(&self) {
        if self.entries.key().is_none() {
            // We won't clear any storage if we are in lazy state since there
            // probably has not been any state written to storage, yet.
            return
        }
        for index in 0..self.len_entries() {
            // It might seem wasteful to clear all entries instead of just
            // the occupied ones. However this spares us from having one extra
            // read for every element in the storage stash to filter out vacant
            // entries. So this is actually a trade-off and at the time of this
            // implementation it is unclear which path is more efficient.
            //
            // The bet is that clearing a storage cell is cheaper than reading one.
            self.entries.clear_packed_at(index);
        }
    }
}

impl<T> Stash<T>
where
    T: PackedLayout,
{
    /// Rebinds the `prev` and `next` bindings of the neighbours of the vacant entry.
    ///
    /// # Note
    ///
    /// The `removed_index` points to the index of the removed vacant entry.
    fn remove_vacant_entry(&mut self, removed_index: Index, vacant_entry: VacantEntry) {
        let prev_vacant = vacant_entry.prev;
        let next_vacant = vacant_entry.next;
        if prev_vacant == removed_index && next_vacant == removed_index {
            // There is no other vacant entry left in the storage stash so
            // there is nothing to update. Bail out early.
            self.header.last_vacant = self.header.len;
            return
        }
        if prev_vacant == next_vacant {
            // There is only one other vacant entry left.
            // We can update the single vacant entry in a single look-up.
            let entry = self
                .entries
                .get_mut(prev_vacant)
                .map(Entry::try_to_vacant_mut)
                .flatten()
                .expect("`prev` must point to an existing entry at this point");
            debug_assert_eq!(entry.prev, removed_index);
            debug_assert_eq!(entry.next, removed_index);
            entry.prev = prev_vacant;
            entry.next = prev_vacant;
        } else {
            // There are multiple other vacant entries left.
            let prev = self
                .entries
                .get_mut(prev_vacant)
                .map(Entry::try_to_vacant_mut)
                .flatten()
                .expect("`prev` must point to an existing entry at this point");
            debug_assert_eq!(prev.next, removed_index);
            prev.next = next_vacant;
            let next = self
                .entries
                .get_mut(next_vacant)
                .map(Entry::try_to_vacant_mut)
                .flatten()
                .expect("`next` must point to an existing entry at this point");
            debug_assert_eq!(next.prev, removed_index);
            next.prev = prev_vacant;
        }
        // Bind the last vacant pointer to the vacant position with the lower index.
        // This has the effect that lower indices are refilled more quickly.
        use core::cmp::min;
        if removed_index == self.header.last_vacant {
            self.header.last_vacant = min(prev_vacant, next_vacant);
        }
    }

    /// Returns the previous and next vacant entry for the entry at index `at`.
    ///
    /// If there exists a last vacant entry, the return value is a tuple
    /// `(index_of_previous_vacant, index_of_next_vacant)`.
    /// The two `index_` values hereby are selected in a way that makes it
    /// more likely that the stash is refilled from low indices.
    ///
    /// If no vacant entry exists a self-referential tuple of `(at, at)`
    /// is returned.
    fn fetch_prev_and_next_vacant_entry(&self, at: Index) -> (Index, Index) {
        if let Some(index) = self.last_vacant_index() {
            let root_vacant = self
                .entries
                .get(index)
                .map(|entry| entry.try_to_vacant())
                .flatten()
                .expect("last_vacant must point to an existing vacant entry");
            // Form the linked vacant entries in a way that makes it more likely
            // for them to refill the stash from low indices.
            if at < index {
                // Insert before root if new vacant index is smaller than root.
                (root_vacant.prev, index)
            } else if at < root_vacant.next {
                // Insert between root and its next vacant entry if smaller than
                // current root's next index.
                (index, root_vacant.next)
            } else {
                // Insert before root entry if index is greater. But we won't
                // update the new element to be the new root index in this case.
                (root_vacant.prev, index)
            }
        } else {
            // Default prev and next to the given at index.
            // So the resulting vacant index is pointing to itself.
            (at, at)
        }
    }

    /// Updates links from and to neighbouring vacant entries.
    fn update_neighboring_vacant_entry_links(
        &mut self,
        prev: Index,
        next: Index,
        at: Index,
    ) {
        if prev == next {
            // Previous and next are the same so we can update the vacant
            // neighbour with a single look-up.
            let entry = self
                .entries
                .get_mut(next)
                .map(Entry::try_to_vacant_mut)
                .flatten()
                .expect("`next` must point to an existing vacant entry at this point");
            entry.prev = at;
            entry.next = at;
        } else {
            // Previous and next vacant entries are different and thus need
            // different look-ups to update them.
            self.entries
                .get_mut(prev)
                .map(Entry::try_to_vacant_mut)
                .flatten()
                .expect("`prev` must point to an existing vacant entry at this point")
                .next = at;
            self.entries
                .get_mut(next)
                .map(Entry::try_to_vacant_mut)
                .flatten()
                .expect("`next` must point to an existing vacant entry at this point")
                .prev = at;
        }
    }

    /// Put the element into the stash at the next vacant position.
    ///
    /// Returns the stash index that the element was put into.
    pub fn put(&mut self, new_value: T) -> Index {
        let new_entry = Some(Entry::Occupied(new_value));
        let new_index = if let Some(index) = self.last_vacant_index() {
            // Put the new element to the most recent vacant index if not all entries are occupied.
            let old_entry = self
                .entries
                .put_get(index, new_entry)
                .expect("a `last_vacant_index()` must point to an occupied cell");
            let vacant_entry = match old_entry {
                Entry::Vacant(vacant_entry) => vacant_entry,
                Entry::Occupied(_) => {
                    unreachable!("`last_vacant_index()` must point to a vacant entry")
                }
            };
            self.remove_vacant_entry(index, vacant_entry);
            index
        } else {
            // Push the new element to the end if all entries are occupied.
            let new_index = self.header.len_entries;
            self.entries.put(new_index, new_entry);
            self.header.last_vacant += 1;
            self.header.len_entries += 1;
            new_index
        };
        self.header.len += 1;
        new_index
    }

    /// Takes the element stored at the given index if any.
    pub fn take(&mut self, at: Index) -> Option<T> {
        // Cases:
        // - There are vacant entries already.
        // - There are no vacant entries before.
        if at >= self.len_entries() {
            // Early return since `at` index is out of bounds.
            return None
        }
        // Precompute prev and next vacant entries as we might need them later.
        // Due to borrow checker constraints we cannot have this at a later stage.
        let (prev, next) = self.fetch_prev_and_next_vacant_entry(at);
        let entry_mut = self.entries.get_mut(at).expect("index is out of bounds");
        if entry_mut.is_vacant() {
            // Early return if the taken entry is already vacant.
            return None
        }
        // At this point we know that the entry is occupied with a value.
        let new_vacant_entry = Entry::Vacant(VacantEntry { prev, next });
        let taken_entry = core::mem::replace(entry_mut, new_vacant_entry);
        self.update_neighboring_vacant_entry_links(prev, next, at);
        // Take the value out of the taken occupied entry and return it.
        match taken_entry {
            Entry::Occupied(value) => {
                use core::cmp::min;
                self.header.last_vacant =
                    min(self.header.last_vacant, min(at, min(prev, next)));
                self.header.len -= 1;
                Some(value)
            }
            Entry::Vacant { .. } => {
                unreachable!("the taken entry is known to be occupied")
            }
        }
    }

    /// Removes the element stored at the given index if any.
    ///
    /// This method acts similar to the take API and even still returns an Option.
    /// However, it guarantees to make no contract storage reads to the indexed
    /// element and will only write to its internal low-level lazy cache that the
    /// element at the given index is going to be removed at the end of the contract
    /// execution.
    ///
    /// Calling this method with an index out of bounds for the returns `None` and
    /// does not `remove` the element, otherwise it returns `Some(())`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `at` refers to an occupied index. Behavior is
    /// unspecified if `at` refers to a vacant index and could seriously damage the
    /// contract storage integrity.
    pub unsafe fn remove_occupied(&mut self, at: Index) -> Option<()> {
        // This function is written similar to [`Stash::take`], with the exception
        // that the caller has to ensure that `at` refers to an occupied entry whereby
        // the procedure can avoid loading the occupied entry which might be handy if
        // the stored `T` is especially costly to load from contract storage.
        if at >= self.len_entries() {
            // Early return since `at` index is out of bounds.
            return None
        }
        // Precompute prev and next vacant entries as we might need them later.
        // Due to borrow checker constraints we cannot have this at a later stage.
        let (prev, next) = self.fetch_prev_and_next_vacant_entry(at);
        let new_vacant_entry = Entry::Vacant(VacantEntry { prev, next });
        self.entries.put(at, Some(new_vacant_entry));
        self.update_neighboring_vacant_entry_links(prev, next, at);
        use core::cmp::min;
        self.header.last_vacant = min(self.header.last_vacant, min(at, min(prev, next)));
        self.header.len -= 1;
        Some(())
    }

    /// Defragments the underlying storage to minimize footprint.
    ///
    /// Returns the number of storage cells freed this way.
    ///
    /// This might invalidate indices stored outside of the stash.
    ///
    /// # Callback
    ///
    /// In order to keep those indices up-to-date the caller can provide
    /// a callback function that is called for every moved entry
    /// with a shared reference to the entries value and the old as well
    /// as the new index.
    ///
    /// # Note
    ///
    /// - If `max_iterations` is `Some` concrete value it is used in order to
    ///   bound the number of iterations and won't try to defrag until the stash
    ///   is optimally compacted.
    /// - Users are advised to call this method using `Some` concrete
    ///   value to keep gas costs within certain bounds.
    /// - The call to the given callback takes place before the reinsertion
    ///   of the shifted occupied entry.
    pub fn defrag<C>(&mut self, max_iterations: Option<u32>, mut callback: C) -> u32
    where
        C: FnMut(Index, Index, &T),
    {
        let len_entries = self.len_entries();
        let mut freed_cells = 0;
        for index in (0..len_entries)
            .rev()
            .take(max_iterations.unwrap_or(len_entries) as usize)
        {
            if !self.has_vacant_entries() {
                // Bail out as soon as there are no more vacant entries left.
                return freed_cells
            }
            // In any case we are going to free yet another storage cell.
            freed_cells += 1;
            match self
                .entries
                .put_get(index, None)
                .expect("index is out of bounds")
            {
                Entry::Vacant(vacant_entry) => {
                    // Remove the vacant entry and rebind its neighbours.
                    self.remove_vacant_entry(index, vacant_entry);
                }
                Entry::Occupied(value) => {
                    // Move the occupied entry into one of the remaining vacant
                    // entries. We do not re-use the `put` method to not update
                    // the length and other header information.
                    let vacant_index = self
                        .last_vacant_index()
                        .expect("it has been asserted that there are vacant entries");
                    callback(index, vacant_index, &value);
                    let new_entry = Some(Entry::Occupied(value));
                    let old_entry = self.entries.put_get(vacant_index, new_entry).expect(
                        "`last_vacant_index` index must point to an occupied cell",
                    );
                    let vacant_entry = match old_entry {
                        Entry::Vacant(vacant_entry) => vacant_entry,
                        Entry::Occupied(_) => {
                            unreachable!(
                                "`last_vacant_index` must point to a vacant entry"
                            )
                        }
                    };
                    self.remove_vacant_entry(vacant_index, vacant_entry);
                }
            }
            self.header.len_entries -= 1;
        }
        freed_cells
    }
}
